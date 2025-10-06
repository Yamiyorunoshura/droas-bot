#!/usr/bin/env python3

"""
shard-requirements.py
將 requirements.md 依二級標題(##)拆分為多個獨立 .md 檔，並確保以 uv 建立的
Python 3.13 虛擬環境於專案根目錄 .venv 下執行。

環境變數：
- REQUIREMENTS_FILE：輸入檔路徑（預設 docs/requirements.md）
- OUTPUT_DIR：輸出資料夾（預設 docs/requirements）
"""

from __future__ import annotations

import os
import re
import sys
import shutil
import subprocess
from pathlib import Path
from typing import Optional


RED = "\033[0;31m"
GREEN = "\033[0;32m"
YELLOW = "\033[1;33m"
BLUE = "\033[0;34m"
NC = "\033[0m"


def log_info(message: str) -> None:
    print(f"{BLUE}[INFO]{NC} {message}")


def log_success(message: str) -> None:
    print(f"{GREEN}[SUCCESS]{NC} {message}")


def log_warning(message: str) -> None:
    print(f"{YELLOW}[WARNING]{NC} {message}")


def log_error(message: str) -> None:
    print(f"{RED}[ERROR]{NC} {message}")


def get_project_root() -> Path:
    # scripts 目錄的上上層即為專案根目錄（跳過 claude code 目錄）
    return Path(__file__).resolve().parents[2]


def ensure_uv_and_venv(python_version: str = "3.13") -> None:
    project_root = get_project_root()
    venv_dir = project_root / ".venv"
    uv_path = shutil.which("uv")

    current_prefix = Path(sys.prefix).resolve()
    if current_prefix == venv_dir.resolve() and sys.version_info.major == 3 and sys.version_info.minor == 13:
        return

    if uv_path is None:
        log_error("未安裝 uv，請先安裝：curl -LsSf https://astral.sh/uv/install.sh | sh")
        sys.exit(1)

    if os.environ.get("SC_BOOTSTRAPPED") == "1":
        return

    try:
        subprocess.run([uv_path, "python", "install", python_version], check=False, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    except Exception:
        pass

    if not venv_dir.exists():
        log_info(f"建立本地虛擬環境：{venv_dir} (Python {python_version})")
        subprocess.run([uv_path, "venv", "--python", python_version, str(venv_dir)], check=True)

    venv_python = venv_dir / ("Scripts/python.exe" if os.name == "nt" else "bin/python")
    if not venv_python.exists():
        log_error(f"找不到 venv Python：{venv_python}")
        sys.exit(1)

    env = os.environ.copy()
    env["SC_BOOTSTRAPPED"] = "1"
    argv = [str(venv_python), str(Path(__file__).resolve()), *sys.argv[1:]]
    os.execve(str(venv_python), argv, env)


def clean_filename(filename: str) -> str:
    filename = re.sub(r"[\\/:*?\"<>|]", "", filename)
    filename = re.sub(r"\s+", " ", filename)
    return filename.strip()


def create_directory_structure(output_dir: Path) -> None:
    log_info("創建目錄結構...")
    output_dir.mkdir(parents=True, exist_ok=True)
    log_success("目錄結構創建完成")


def check_input_file(input_file: Path) -> None:
    if not input_file.is_file():
        log_error(f"找不到輸入文件: {input_file}")
        sys.exit(1)
    if not os.access(str(input_file), os.R_OK):
        log_error(f"無法讀取輸入文件: {input_file}")
        sys.exit(1)
    log_success(f"輸入文件檢查通過: {input_file}")


def extract_second_level_headers(input_file: Path, output_dir: Path) -> int:
    log_info("提取二級標題...")
    sections_count = 0
    current_section_title: Optional[str] = None
    current_section_path: Optional[Path] = None

    with input_file.open("r", encoding="utf-8") as f:
        for raw_line in f:
            line = raw_line.rstrip("\n")
            m = re.match(r"^##\s+(.+)$", line)
            if m:
                section_title = m.group(1)

                if current_section_title and current_section_path:
                    log_info(f"已保存: {current_section_path}")
                    sections_count += 1

                current_section_title = section_title
                filename = clean_filename(section_title) + ".md"
                current_section_path = output_dir / filename
                with current_section_path.open("w", encoding="utf-8") as out:
                    out.write(f"## {section_title}\n\n")
                continue

            if current_section_path is not None:
                with current_section_path.open("a", encoding="utf-8") as out:
                    out.write(raw_line)

    if current_section_title and current_section_path:
        log_info(f"已保存: {current_section_path}")
        sections_count += 1

    log_success(f"二級標題提取完成，共 {sections_count} 個章節")
    return sections_count


def list_md_files_recursively(directory: Path) -> list[Path]:
    return sorted(p for p in directory.rglob("*.md") if p.is_file())


def list_md_files_top_level(directory: Path) -> list[Path]:
    return sorted(p for p in directory.glob("*.md") if p.is_file())


def generate_report(input_file: Path, output_dir: Path) -> None:
    log_info("生成處理報告...")

    print("")
    print("==============================================")
    print("         需求文檔拆分完成報告")
    print("==============================================")
    print(f"輸入文件: {input_file}")
    print(f"輸出目錄: {output_dir}")
    print("")
    print("生成的文件結構:")
    for path in list_md_files_recursively(output_dir):
        rel = path.relative_to(output_dir)
        print(str(rel))
    print("")
    print("文件統計:")
    total_top = len(list_md_files_top_level(output_dir))
    total_all = len(list_md_files_recursively(output_dir))
    print(f"- 二級標題文件: {total_top}")
    print(f"- 總文件數: {total_all}")
    print("==============================================")


def main() -> None:
    ensure_uv_and_venv("3.13")

    project_root = get_project_root()
    input_file_env = os.environ.get("REQUIREMENTS_FILE", "docs/requirements.md")
    output_dir_env = os.environ.get("OUTPUT_DIR", "docs/requirements")

    input_file = Path(input_file_env)
    if not input_file.is_absolute():
        input_file = (project_root / input_file_env).resolve()

    output_dir = Path(output_dir_env)
    if not output_dir.is_absolute():
        output_dir = (project_root / output_dir_env).resolve()

    check_input_file(input_file)
    create_directory_structure(output_dir)
    extract_second_level_headers(input_file, output_dir)
    generate_report(input_file, output_dir)
    log_success("需求文檔拆分完成！")
    
    # 刪除原始文檔
    if input_file.exists():
        log_info(f"刪除原始文檔: {input_file}")
        input_file.unlink()
        log_success("原始文檔已刪除")


if __name__ == "__main__":
    main()


