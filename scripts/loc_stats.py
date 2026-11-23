#!/usr/bin/env python3
import os
import sys

def main():
    root_dir = os.path.abspath(os.path.dirname(os.path.dirname(__file__))) if "scripts" in __file__ else os.path.abspath(".")
    crates_dir = os.path.join(root_dir, "crates")

    if not os.path.exists(crates_dir):
        print(f"Error: {crates_dir} not found")
        sys.exit(1)

    crate_stats = {}
    file_records = []

    total_files = 0
    total_lines = 0
    total_code = 0
    total_comments = 0
    total_blank = 0
    total_chars = 0
    total_chars_nospace = 0
    total_bytes = 0

    for crate_name in sorted(os.listdir(crates_dir)):
        crate_path = os.path.join(crates_dir, crate_name)
        if not os.path.isdir(crate_path) or crate_name == "agent-hub":
            continue

        c_files = 0
        c_lines = 0
        c_code = 0
        c_comments = 0
        c_blank = 0
        c_chars = 0
        c_chars_nospace = 0
        c_bytes = 0

        for dirpath, _, filenames in os.walk(crate_path):
            for f in sorted(filenames):
                if f.endswith(".rs"):
                    file_path = os.path.join(dirpath, f)
                    rel_path = os.path.relpath(file_path, root_dir)
                    c_files += 1

                    f_lines = 0
                    f_code = 0
                    f_comments = 0
                    f_blank = 0
                    f_chars = 0
                    f_chars_nospace = 0
                    f_bytes = 0

                    try:
                        f_bytes = os.path.getsize(file_path)
                        with open(file_path, "r", encoding="utf-8", errors="ignore") as fp:
                            content = fp.read()
                            f_chars = len(content)
                            f_chars_nospace = len("".join(content.split()))

                        lines = content.splitlines()
                        in_block_comment = False
                        for line in lines:
                            f_lines += 1
                            sline = line.strip()
                            if not sline:
                                f_blank += 1
                            elif in_block_comment:
                                f_comments += 1
                                if "*/" in sline:
                                    in_block_comment = False
                            elif sline.startswith("/*"):
                                f_comments += 1
                                if "*/" not in sline:
                                    in_block_comment = True
                            elif sline.startswith("//"):
                                f_comments += 1
                            else:
                                f_code += 1

                    except Exception as e:
                        pass

                    file_records.append({
                        "crate": crate_name,
                        "rel_path": rel_path,
                        "lines": f_lines,
                        "code": f_code,
                        "comments": f_comments,
                        "blank": f_blank,
                        "chars": f_chars,
                        "chars_nospace": f_chars_nospace,
                        "bytes": f_bytes,
                    })

                    c_lines += f_lines
                    c_code += f_code
                    c_comments += f_comments
                    c_blank += f_blank
                    c_chars += f_chars
                    c_chars_nospace += f_chars_nospace
                    c_bytes += f_bytes

        crate_stats[crate_name] = {
            "files": c_files,
            "lines": c_lines,
            "code": c_code,
            "comments": c_comments,
            "blank": c_blank,
            "chars": c_chars,
            "chars_nospace": c_chars_nospace,
            "bytes": c_bytes,
        }

        total_files += c_files
        total_lines += c_lines
        total_code += c_code
        total_comments += c_comments
        total_blank += c_blank
        total_chars += c_chars
        total_chars_nospace += c_chars_nospace
        total_bytes += c_bytes

    print("=" * 110)
    print("                     ПОЛНАЯ ДВУХКАТЕГОРИЙНАЯ СТАТИСТИКА OMNIGRAM-RS (БЕЗ AGENT-HUB)")
    print("=" * 110)

    # 1. СТАТИСТИКА ПО СТРОКАМ
    print("\n>>> КАТЕГОРИЯ 1: СТАТИСТИКА ПО СТРОКАМ (КРЕЙТЫ)")
    h_lines = f"{'Крейт':<24} | {'Файлов':>6} | {'Чистый код':>11} | {'Комментарии':>11} | {'Пустые':>7} | {'Всего строк':>11} | {'% Кода':>7}"
    print(h_lines)
    print("-" * len(h_lines))
    for crate, s in sorted(crate_stats.items(), key=lambda x: x[1]['lines'], reverse=True):
        code_pct = (s['code'] / s['lines'] * 100) if s['lines'] else 0
        print(f"{crate:<24} | {s['files']:>6} | {s['code']:>11} | {s['comments']:>11} | {s['blank']:>7} | {s['lines']:>11} | {code_pct:>6.1f}%")
    print("=" * len(h_lines))
    tot_code_pct = (total_code / total_lines * 100) if total_lines else 0
    print(f"{'ИТОГО':<24} | {total_files:>6} | {total_code:>11} | {total_comments:>11} | {total_blank:>7} | {total_lines:>11} | {tot_code_pct:>6.1f}%")

    # ТОП файлов по строкам
    print("\n--- ТОП-10 САМЫХ БОЛЬШИХ ФАЙЛОВ ПО СТРОКАМ ---")
    by_lines = sorted(file_records, key=lambda x: x['lines'], reverse=True)[:10]
    h_top_l = f"{'#':<3} | {'Строк':>7} | {'Код':>7} | {'Коммент':>7} | {'Файл'}"
    print(h_top_l)
    print("-" * 90)
    for idx, f in enumerate(by_lines, 1):
        print(f"{idx:<3} | {f['lines']:>7} | {f['code']:>7} | {f['comments']:>7} | {f['rel_path']}")

    # 2. СТАТИСТИКА ПО СИМВОЛАМ И ОБЪЁМУ
    print("\n\n>>> КАТЕГОРИЯ 2: СТАТИСТИКА ПО СИМВОЛАМ И БАЙТАМ (КРЕЙТЫ)")
    h_chars = f"{'Крейт':<24} | {'Символов всего':>14} | {'Символов (без пробелов)':>23} | {'Размер (КБ)':>11} | {'Симв/строку':>11}"
    print(h_chars)
    print("-" * len(h_chars))
    for crate, s in sorted(crate_stats.items(), key=lambda x: x[1]['chars'], reverse=True):
        kb = s['bytes'] / 1024.0
        cps = (s['chars'] / s['lines']) if s['lines'] else 0
        print(f"{crate:<24} | {s['chars']:>14,}".replace(",", " ") + f" | {s['chars_nospace']:>23,}".replace(",", " ") + f" | {kb:>10.2f}  | {cps:>11.1f}")
    print("=" * len(h_chars))
    tot_kb = total_bytes / 1024.0
    tot_cps = (total_chars / total_lines) if total_lines else 0
    print(f"{'ИТОГО':<24} | {total_chars:>14,}".replace(",", " ") + f" | {total_chars_nospace:>23,}".replace(",", " ") + f" | {tot_kb:>10.2f}  | {tot_cps:>11.1f}")

    # ТОП файлов по символам
    print("\n--- ТОП-10 САМЫХ БОЛЬШИХ ФАЙЛОВ ПО СИМВОЛАМ ---")
    by_chars = sorted(file_records, key=lambda x: x['chars'], reverse=True)[:10]
    h_top_c = f"{'#':<3} | {'Символов':>10} | {'Без пробелов':>12} | {'Размер':>9} | {'Файл'}"
    print(h_top_c)
    print("-" * 90)
    for idx, f in enumerate(by_chars, 1):
        kb_str = f"{f['bytes'] / 1024.0:.1f} КБ"
        print(f"{idx:<3} | {f['chars']:>10,} | {f['chars_nospace']:>12,} | {kb_str:>9} | {f['rel_path']}".replace(",", " "))

    # РАСПРЕДЕЛЕНИЕ СИМВОЛОВ И СТРОК
    print("\n--- СВОДНЫЕ МЕТРИКИ И ЭКСТРЕМУМЫ ---")
    max_line_file = by_lines[0]
    max_char_file = by_chars[0]
    min_line_file = min(file_records, key=lambda x: x['lines'])
    min_char_file = min(file_records, key=lambda x: x['chars'])

    print(f"• Всего файлов .rs: {total_files}")
    print(f"• Всего символов (с пробелами): {total_chars:,} ({total_chars_nospace:,} без пробельных символов)".replace(",", " "))
    print(f"• Всего строк: {total_lines:,} (из них чистого кода: {total_code:,}, комментариев: {total_comments:,}, пустых: {total_blank:,})".replace(",", " "))
    print(f"• Средняя длина строки: {tot_cps:.2f} символов")
    print(f"• Самый большой файл ПО СТРОКАМ:   {max_line_file['rel_path']} ({max_line_file['lines']:,} строк, {max_line_file['code']:,} кода)".replace(",", " "))
    print(f"• Самый большой файл ПО СИМВОЛАМ:  {max_char_file['rel_path']} ({max_char_file['chars']:,} симв., {max_char_file['bytes']/1024:.1f} КБ)".replace(",", " "))
    print(f"• Самый компактный файл:          {min_line_file['rel_path']} ({min_line_file['lines']} строк, {min_line_file['chars']} симв.)")
    print("=" * 110)

if __name__ == "__main__":
    main()
