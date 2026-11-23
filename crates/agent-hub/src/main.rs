//! agent-hub — CLI-координатор мульти-агентной работы над портом tdesktop на Rust.
//!
//! Команды:
//!   agent-hub claim    --id <task> --agent <name>   — взять задачу (блокирует related)
//!   agent-hub release  --id <task>                  — снять лок (при ошибке/прерывании)
//!   agent-hub complete --id <task>                  — сдать задачу (разблокирует граф)
//!   agent-hub status                                — дерево графа задач
//!   agent-hub next     --agent <name>               — следующая свободная задача

mod lock;
mod map;
mod state;

use map::FileMap;
use state::{State, Status, TaskState};
use std::path::Path;

const MAP_PATH: &str = "file_map.json";
const STATE_PATH: &str = ".agent-state.json";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(e) = run(&args) {
        eprintln!("Ошибка: {e}");
        std::process::exit(1);
    }
}

// ---------------------------------------------------------------- CLI-парсер

struct Cli {
    cmd: String,
    id: Option<String>,
    agent: Option<String>,
}

fn parse_cli(args: &[String]) -> Result<Cli, String> {
    if args.is_empty() {
        return Err(usage());
    }
    let mut cli = Cli { cmd: args[0].clone(), id: None, agent: None };
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--id" => cli.id = Some(val(args, &mut i)),
            "--agent" => cli.agent = Some(val(args, &mut i)),
            other => return Err(format!("неизвестный аргумент `{other}`\n{}", usage())),
        }
        i += 1;
    }
    Ok(cli)
}

fn val(args: &[String], i: &mut usize) -> String {
    *i += 1;
    args.get(*i).cloned().unwrap_or_default()
}

fn usage() -> String {
    "использование:\n  \
     agent-hub claim --id <task> --agent <name>\n  \
     agent-hub release --id <task>\n  \
     agent-hub complete --id <task>\n  \
     agent-hub status\n  \
     agent-hub next --agent <name>"
        .into()
}

// ------------------------------------------------------------------- Run

fn run(args: &[String]) -> Result<(), String> {
    let cli = parse_cli(args)?;

    let map = FileMap::load(Path::new(MAP_PATH))?;

    let mut guard = lock::StateGuard::acquire(Path::new(STATE_PATH))
        .map_err(|e| format!("не удалось захватить flock: {e}"))?;
    let mut st = State::parse(&guard.read().map_err(|e| e.to_string())?)?;

    match cli.cmd.as_str() {
        "claim" => claim(&map, &mut st, need(&cli.id, "--id")?, need(&cli.agent, "--agent")?)?,
        "release" => release(&map, &mut st, need(&cli.id, "--id")?)?,
        "complete" => complete(&map, &mut st, need(&cli.id, "--id")?)?,
        "status" => {
            print_status(&map, &st);
            return Ok(()); // read-only, ничего не пишем
        }
        "next" => {
            let agent = need(&cli.agent, "--agent")?;
            match next_task(&map, &st) {
                Some(t) => {
                    // Сразу claim от имени агента: next = атомарные next+claim.
                    claim(&map, &mut st, &t.id, &agent)?;
                    println!("{} (приоритет {})", t.id, t.priority);
                    if !t.paths.is_empty() {
                        println!("пути: {}", t.paths.join(", "));
                    }
                    let rel = related_of(&map, &t.id);
                    if !rel.is_empty() {
                        println!("заблокировано за вами: {}", rel.join(", "));
                    }
                }
                None => println!("свободных незаблокированных задач нет"),
            }
        }
        other => return Err(format!("неизвестная команда `{other}`\n{}", usage())),
    }

    let json = serde_json::to_string_pretty(&st).map_err(|e| e.to_string())?;
    guard
        .write_atomic(Path::new(STATE_PATH), &json)
        .map_err(|e| format!("не удалось записать состояние: {e}"))?;
    Ok(())
}

fn need<'a>(v: &'a Option<String>, flag: &str) -> Result<&'a str, String> {
    v.as_deref().filter(|s| !s.is_empty()).ok_or(format!("не указан {flag}\n{}", usage()))
}

fn related_of(map: &FileMap, id: &str) -> Vec<String> {
    let mut rel = map.task(id).map(|t| t.related.clone()).unwrap_or_default();
    for imp in map.implicit_related(id) {
        if !rel.contains(&imp) {
            rel.push(imp);
        }
    }
    rel
}

// ----------------------------------------------------------------- Команды

fn claim(map: &FileMap, st: &mut State, id: &str, agent: &str) -> Result<(), String> {
    let task = map.task(id).ok_or(format!("задача `{id}` отсутствует в file_map.json"))?;

    // Related-лок: чужой держатель запрещает claim.
    if let Some(ts) = st.tasks.get(id) {
        if ts.locked && ts.agent.as_deref() != Some(agent) {
            return Err(format!(
                "задача `{id}` заблокирована related-локом агента {}",
                ts.agent.as_deref().unwrap_or("?")
            ));
        }
    }

    let deps_bad: Vec<&String> = task
        .depends_on
        .iter()
        .filter(|d| st.status_of(d) != Status::Done)
        .collect();
    if !deps_bad.is_empty() {
        return Err(format!(
            "задача `{id}` заблокирована незавершёнными зависимостями: {}",
            deps_bad.iter().map(|d| d.as_str()).collect::<Vec<_>>().join(", ")
        ));
    }

    let cur = st.status_of(id);
    if cur == Status::InProgress {
        return Err(format!(
            "задача `{id}` уже в работе (агент {})",
            st.agent_of(id).unwrap_or_default()
        ));
    }
    if cur == Status::Done {
        return Err(format!("задача `{id}` уже сдана"));
    }

    let related = related_of(map, id);
    let conflicts: Vec<&String> = related
        .iter()
        .filter(|r| {
            st.status_of(r) == Status::InProgress
                || st
                    .tasks
                    .get(*r)
                    .map(|ts| ts.locked && ts.agent.as_deref() != Some(agent))
                    .unwrap_or(false)
        })
        .collect();
    if !conflicts.is_empty() {
        return Err(format!(
            "связанные задачи заняты другими агентами: {}",
            conflicts.iter().map(|d| d.as_str()).collect::<Vec<_>>().join(", ")
        ));
    }

    // Лочим саму задачу; связанные помечаем related-локом (НЕ "в работе").
    st.tasks.insert(
        id.to_string(),
        TaskState { status: Status::InProgress, agent: Some(agent.to_string()), locked: false },
    );
    for r in &related {
        let entry = st.tasks.entry(r.clone()).or_insert_with(|| TaskState {
            status: Status::Pending,
            agent: None,
            locked: false,
        });
        if entry.status == Status::Pending {
            entry.locked = true;
            entry.agent = Some(agent.to_string());
        }
    }
    println!("ок: `{id}` закреплена за {agent}");
    Ok(())
}

fn release(map: &FileMap, st: &mut State, id: &str) -> Result<(), String> {
    match st.tasks.get(id) {
        None => Err(format!("задача `{id}` не значится в состоянии")),
        Some(t) if t.status != Status::InProgress => Err(format!("задача `{id}` не в работе")),
        Some(t) => {
            let agent = t.agent.clone();
            // Снимаем related-локи, которые держал этот же агент.
            let to_free: Vec<String> = related_of(map, id)
                .into_iter()
                .filter(|r| {
                    st.tasks
                        .get(r)
                        .map(|ts| ts.locked && ts.agent == agent)
                        .unwrap_or(false)
                })
                .collect();
            st.tasks.remove(id);
            for r in &to_free {
                if let Some(ts) = st.tasks.get_mut(r) {
                    ts.locked = false;
                    ts.agent = None;
                }
            }
            println!("ок: `{id}` возвращена в пул");
            if !to_free.is_empty() {
                println!("разблокировано: {}", to_free.join(", "));
            }
            Ok(())
        }
    }
}

fn complete(map: &FileMap, st: &mut State, id: &str) -> Result<(), String> {
    match st.tasks.get(id).cloned() {
        None => Err(format!("задача `{id}` не значится в состоянии (сначала claim)")),
        Some(t) if t.status != Status::InProgress => Err(format!("задача `{id}` не в работе")),
        Some(_) => {
            // Снимаем related-локи, закреплённые этим же агентом.
            let agent = st.agent_of(id);
            let unlocked: Vec<String> = st
                .tasks
                .iter()
                .filter(|(_, ts)| ts.locked && ts.agent == agent)
                .map(|(k, _)| k.clone())
                .collect();
            st.tasks.insert(
                id.to_string(),
                TaskState { status: Status::Done, agent: st.agent_of(id), locked: false },
            );
            for k in &unlocked {
                if k != id {
                    let ts = st.tasks.get_mut(k).unwrap();
                    ts.locked = false;
                    ts.agent = None;
                }
            }
            // Какие задачи разблокировались?
            let now_free: Vec<&String> = map
                .tasks
                .iter()
                .filter(|t| t.depends_on.iter().any(|d| d == id))
                .map(|t| &t.id)
                .collect();
            println!("ок: `{id}` сдана");
            if !now_free.is_empty() {
                println!("разблокировано: {}", now_free.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", "));
            }
            Ok(())
        }
    }
}

fn next_task<'a>(map: &'a FileMap, st: &State) -> Option<&'a map::Task> {
    map.tasks
        .iter()
        .filter(|t| {
            let s = st.status_of(&t.id);
            s == Status::Pending
        })
        .filter(|t| {
            st.tasks
                .get(&t.id)
                .map(|ts| !ts.locked)
                .unwrap_or(true)
        })
        .filter(|t| t.depends_on.iter().all(|d| st.status_of(d) == Status::Done))
        .filter(|t| {
            related_of(map, &t.id)
                .iter()
                .all(|r| st.status_of(r) != Status::InProgress)
        })
        .min_by_key(|t| t.priority) // 1 — фундамент, идёт первым
}

fn print_status(map: &FileMap, st: &State) {
    let mut by_prio: Vec<&map::Task> = map.tasks.iter().collect();
    by_prio.sort_by_key(|t| (t.priority, t.id.clone()));

    let names = [(1, "фундамент"), (2, "сеть/хранилище"), (3, "модели данных"),
                 (4, "стили/ресурсы"), (5, "UI")];

    for prio in [1u32, 2, 3, 4, 5] {
        let group: Vec<_> = by_prio.iter().filter(|t| t.priority == prio).collect();
        if group.is_empty() {
            continue;
        }
        println!("[{prio}] {}", names[prio as usize - 1].1);
        for t in group {
            let s = st.status_of(&t.id);
            let agent = st.agent_of(&t.id).unwrap_or_default();
            let mark = match s {
                Status::Done => "[✔]",
                Status::InProgress => "[▶]",
                Status::Pending => {
                    let locked = st.tasks.get(&t.id).map(|ts| ts.locked).unwrap_or(false);
                    let ready = t.depends_on.iter().all(|d| st.status_of(d) == Status::Done);
                    if locked {
                        "[L]"
                    } else if ready {
                        "[*]"
                    } else {
                        "[ ]"
                    }
                }
            };
            let deps = if t.depends_on.is_empty() {
                String::new()
            } else {
                format!("  (после: {})", t.depends_on.join(", "))
            };
            let who = if agent.is_empty() { String::new() } else { format!(" ← {agent}") };
            println!("  {mark} {}{deps}{who}", t.id);
        }
    }
}
