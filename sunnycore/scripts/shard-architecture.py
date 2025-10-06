#!/usr/bin/env python3

"""
shard-architecture.py
將 architecture.md 文檔依二級標題(##)拆分為多個獨立 .md 檔
並在執行前確保以 uv 建立的 Python 3.13 虛擬環境於專案根目錄下的 .venv。

環境變數：
- ARCHITECTURE_FILE：輸入檔路徑（預設 docs/architecture.md）
- OUTPUT_DIR：輸出資料夾（預設 docs/architecture）
"""

from __future__ import annotations

import os
import re
import sys
import shutil
import subprocess
from pathlib import Path
from typing import Optional


# 顏色定義
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
    """確保存在 uv 與本地 .venv（使用指定 Python 版本）。

    若當前解譯器不在預期的 .venv 中，建立後以該 venv 的 Python 重新執行本腳本。
    """

    project_root = get_project_root()
    venv_dir = project_root / ".venv"
    uv_path = shutil.which("uv")

    # 若已在目標 venv 中且 Python 版本符合，直接返回
    current_prefix = Path(sys.prefix).resolve()
    if current_prefix == venv_dir.resolve() and sys.version_info.major == 3 and sys.version_info.minor == 13:
        return

    if uv_path is None:
        log_error(
            "未安裝 uv，請先安裝：curl -LsSf https://astral.sh/uv/install.sh | sh"
        )
        sys.exit(1)

    if os.environ.get("SC_BOOTSTRAPPED") == "1":
        # 已嘗試過重啟，避免循環
        return

    # 確保指定版本的 Python 可用（若已存在則快返）
    try:
        subprocess.run([uv_path, "python", "install", python_version], check=False, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    except Exception:
        pass

    # 建立本地 venv
    if not venv_dir.exists():
        log_info(f"建立本地虛擬環境：{venv_dir} (Python {python_version})")
        subprocess.run([uv_path, "venv", "--python", python_version, str(venv_dir)], check=True)

    # 以新 venv 的 Python 重新執行本腳本
    venv_python = venv_dir / ("Scripts/python.exe" if os.name == "nt" else "bin/python")
    if not venv_python.exists():
        log_error(f"找不到 venv Python：{venv_python}")
        sys.exit(1)

    env = os.environ.copy()
    env["SC_BOOTSTRAPPED"] = "1"
    argv = [str(venv_python), str(Path(__file__).resolve()), *sys.argv[1:]]
    os.execve(str(venv_python), argv, env)


def clean_filename(filename: str) -> str:
    # 移除 / \ : * ? " < > | 等不合法字元，壓縮多空白，去除前後空白
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


def extract_architecture_sections(input_file: Path, output_dir: Path) -> int:
    log_info("以二級標題(##)切割 architecture.md 文檔...")
    sections_count = 0
    current_section_title: Optional[str] = None
    current_section_path: Optional[Path] = None

    with input_file.open("r", encoding="utf-8") as f:
        for raw_line in f:
            line = raw_line.rstrip("\n")
            m = re.match(r"^##\s+(.+)$", line)
            if m:
                section_title = m.group(1)
                # 完成前一段
                if current_section_title and current_section_path:
                    log_info(f"已保存: {current_section_path}")
                    sections_count += 1

                # 開始新段
                current_section_title = section_title
                filename = clean_filename(section_title) + ".md"
                current_section_path = output_dir / filename
                with current_section_path.open("w", encoding="utf-8") as out:
                    out.write(f"## {section_title}\n\n")
                continue

            # 累加內容
            if current_section_path is not None:
                with current_section_path.open("a", encoding="utf-8") as out:
                    out.write(raw_line)

    # 保存最後一段
    if current_section_title and current_section_path:
        log_info(f"已保存: {current_section_path}")
        sections_count += 1

    log_success(f"架構文檔各節識別完成，共 {sections_count} 個節")
    return sections_count


def list_md_files_recursively(directory: Path) -> list[Path]:
    return sorted(p for p in directory.rglob("*.md") if p.is_file())


def list_md_files_top_level(directory: Path) -> list[Path]:
    return sorted(p for p in directory.glob("*.md") if p.is_file())


def generate_report(input_file: Path, output_dir: Path) -> None:
    log_info("生成處理報告...")

    print("")
    print("==============================================")
    print("         架構文檔拆分完成報告")
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
    total = len(list_md_files_recursively(output_dir))
    print(f"- 總文件數: {total}")
    print("")
    print("文件列表:")
    for path in list_md_files_top_level(output_dir):
        print(path.stem)
    print("==============================================")


def main() -> None:
    # 先確保 uv + venv (Python 3.13)
    ensure_uv_and_venv("3.13")

    project_root = get_project_root()
    input_file_env = os.environ.get("ARCHITECTURE_FILE", "docs/architecture.md")
    output_dir_env = os.environ.get("OUTPUT_DIR", "docs/architecture")

    input_file = Path(input_file_env)
    if not input_file.is_absolute():
        input_file = (project_root / input_file_env).resolve()

    output_dir = Path(output_dir_env)
    if not output_dir.is_absolute():
        output_dir = (project_root / output_dir_env).resolve()

    check_input_file(input_file)
    create_directory_structure(output_dir)
    extract_architecture_sections(input_file, output_dir)
    generate_report(input_file, output_dir)
    log_success("架構文檔拆分完成！")
    
    # 刪除原始文檔
    if input_file.exists():
        log_info(f"刪除原始文檔: {input_file}")
        input_file.unlink()
        log_success("原始文檔已刪除")


if __name__ == "__main__":
    main()


