#!/bin/bash

# shard-architecture.sh
# 將architecture.md文檔拆分為多個獨立文檔的腳本
# 作者: SunnyCore System
# 版本: 1.0.0

set -e  # 遇到錯誤立即退出

# 顏色定義
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 配置變量
ARCHITECTURE_FILE="docs/architecture.md"
OUTPUT_DIR="docs/architecture"

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
    # 移除冒號和特殊字符：/ \ : * ? " < > |
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
    if [[ ! -f "$ARCHITECTURE_FILE" ]]; then
        log_error "找不到輸入文件: $ARCHITECTURE_FILE"
        exit 1
    fi
    
    if [[ ! -r "$ARCHITECTURE_FILE" ]]; then
        log_error "無法讀取輸入文件: $ARCHITECTURE_FILE"
        exit 1
    fi
    
    log_success "輸入文件檢查通過: $ARCHITECTURE_FILE"
}

# 識別並提取架構文檔各節
extract_architecture_sections() {
    log_info "識別architecture.md文檔內容..."
    
    local in_yaml_frontmatter=false
    local in_section=false
    local current_section=""
    local section_content=""
    local sections_count=0
    local yaml_count=0
    
    while IFS= read -r line; do
        # 處理YAML前置元數據
        if [[ $line =~ ^---$ ]]; then
            if [[ $in_yaml_frontmatter == false ]]; then
                in_yaml_frontmatter=true
                ((yaml_count++))
                log_info "開始處理YAML前置元數據"
                continue
            elif [[ $in_yaml_frontmatter == true ]]; then
                in_yaml_frontmatter=false
                log_info "YAML前置元數據處理完成"
                continue
            fi
        fi
        
        # 跳過YAML前置元數據
        if [[ $in_yaml_frontmatter == true ]]; then
            continue
        fi
        
        # 檢查是否是節標題（以冒號結尾的行，且不在代碼塊中）
        if [[ $line =~ ^[[:space:]]*[A-Za-z][A-Za-z0-9[:space:]]*:[[:space:]]*$ ]]; then
            # 保存前一個節
            if [[ $in_section == true && -n "$current_section" ]]; then
                save_architecture_section "$current_section" "$section_content"
                ((sections_count++))
            fi
            
            # 開始新節
            current_section=$(echo "$line" | sed 's/^[[:space:]]*//; s/:[[:space:]]*$//')
            section_content="# $current_section\n\n"
            in_section=true
            log_info "識別到節標題: $current_section"
            continue
        fi
        
        # 添加內容到當前節
        if [[ $in_section == true ]]; then
            section_content+="$line\n"
        fi
        
    done < "$ARCHITECTURE_FILE"
    
    # 保存最後一個節
    if [[ $in_section == true && -n "$current_section" ]]; then
        save_architecture_section "$current_section" "$section_content"
        ((sections_count++))
    fi
    
    log_success "架構文檔各節識別完成，共 $sections_count 個節"
}

# 保存架構節文件
save_architecture_section() {
    local section_title="$1"
    local section_content="$2"
    
    local filename=$(clean_filename "$section_title")
    local filepath="$OUTPUT_DIR/${filename}.md"
    
    echo -e "$section_content" > "$filepath"
    log_info "已保存: $filepath"
}

# 生成處理報告
generate_report() {
    log_info "生成處理報告..."
    
    echo ""
    echo "=============================================="
    echo "         架構文檔拆分完成報告"
    echo "=============================================="
    echo "輸入文件: $ARCHITECTURE_FILE"
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
    echo "- 總文件數: $(find "$OUTPUT_DIR" -name "*.md" | wc -l)"
    echo ""
    echo "文件列表:"
    find "$OUTPUT_DIR" -name "*.md" -exec basename {} .md \; | sort
    echo "=============================================="
}

# 主函數
main() {
    log_info "開始執行架構文檔拆分腳本..."
    
    # 檢查輸入
    check_input_file
    
    # 創建目錄結構
    create_directory_structure
    
    # 提取架構文檔各節
    extract_architecture_sections
    
    # 生成報告
    generate_report
    
    log_success "架構文檔拆分完成！"
}

# 執行主函數
main "$@"