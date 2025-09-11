#!/usr/bin/env bash

# shard-requirements.sh
# 將requirements.md文檔拆分為多個獨立文檔的腳本
# 作者: SunnyCore System
# 版本: 1.0.0

set -euo pipefail  # 嚴格模式：遇到錯誤立即退出、未定義變數報錯、管線錯誤傳遞

# 顏色定義
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 配置變數
REQUIREMENTS_FILE="${REQUIREMENTS_FILE:-docs/requirements.md}"
OUTPUT_DIR="${OUTPUT_DIR:-docs/requirements}"

# 日誌函數
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 清理文件名函數 - 移除不合法的文件名字符
clean_filename() {
    local filename="$1"
    # 移除或替換特殊字符：/ \ : * ? " < > |
    echo "$filename" | sed 's/[\/\\:*?"<>|]//g' | sed 's/  */ /g' | sed 's/^ *//; s/ *$//'
}

# 創建目錄結構
create_directory_structure() {
    log_info "創建目錄結構..."
    
    # 創建主目錄
    mkdir -p "$OUTPUT_DIR"
    
    log_success "目錄結構創建完成"
}

# 檢查輸入文件
check_input_file() {
    if [[ ! -f "$REQUIREMENTS_FILE" ]]; then
        log_error "找不到輸入文件: $REQUIREMENTS_FILE"
        exit 1
    fi
    
    if [[ ! -r "$REQUIREMENTS_FILE" ]]; then
        log_error "無法讀取輸入文件: $REQUIREMENTS_FILE"
        exit 1
    fi
    
    log_success "輸入文件檢查通過: $REQUIREMENTS_FILE"
}

# 提取二級標題
extract_second_level_headers() {
    log_info "提取二級標題..."
    
    local current_section=""
    local current_section_file=""
    local sections_count=0
    
    while IFS= read -r line; do
        # 檢查是否是二級標題
        if [[ $line =~ ^##[[:space:]]+(.+)$ ]]; then
            local section_title="${BASH_REMATCH[1]}"
            log_info "調試：找到二級標題：$section_title"
            
            # 保存前一個章節
            if [[ -n "$current_section" ]]; then
                finalize_second_level_section "$current_section" "$current_section_file"
                sections_count=$((sections_count + 1))
            fi
            
            # 開始新章節
            current_section="$section_title"
            local filename=$(clean_filename "$section_title")
            current_section_file="$OUTPUT_DIR/${filename}.md"
            echo "## $section_title" > "$current_section_file"
            echo "" >> "$current_section_file"
            continue
        fi
        
        # 添加內容到當前章節
        if [[ -n "$current_section" ]]; then
            echo "$line" >> "$current_section_file"
        fi
        
    done < "$REQUIREMENTS_FILE"
    
    # 保存最後一個章節
    if [[ -n "$current_section" ]]; then
        finalize_second_level_section "$current_section" "$current_section_file"
        sections_count=$((sections_count + 1))
    fi
    
    log_success "二級標題提取完成，共 $sections_count 個章節"
    log_info "調試：extract_second_level_headers函數執行完畢"
}

# 完成二級標題文件保存
finalize_second_level_section() {
    local section_title="$1"
    local filepath="$2"
    
    log_info "調試：正在完成章節：$section_title"
    log_info "已保存: $filepath"
    log_info "調試：finalize_second_level_section函數完成"
}



# 生成處理報告
generate_report() {
    log_info "生成處理報告..."
    
    echo ""
    echo "=============================================="
    echo "         需求文檔拆分完成報告"
    echo "=============================================="
    echo "輸入文件: $REQUIREMENTS_FILE"
    echo "輸出目錄: $OUTPUT_DIR"
    echo ""
    echo "生成的文件結構:"
    if command -v tree >/dev/null 2>&1; then
        tree "$OUTPUT_DIR"
    else
        find "$OUTPUT_DIR" -type f -name "*.md" | sort
    fi
    echo ""
    echo "文件統計:"
    echo "- 二級標題文件: $(find "$OUTPUT_DIR" -maxdepth 1 -name "*.md" | wc -l)"
    echo "- 總文件數: $(find "$OUTPUT_DIR" -name "*.md" | wc -l)"
    echo "=============================================="
}

# 主函數
main() {
    log_info "開始執行需求文檔拆分腳本..."
    
    # 檢查輸入
    check_input_file
    
    # 創建目錄結構
    create_directory_structure
    
    # 提取各部分內容（統一以二級標題切割）
    extract_second_level_headers
    
    # 生成報告
    generate_report
    
    log_success "需求文檔拆分完成！"
}

# 執行主函數
main "$@"