#!/bin/bash

# shard-requirements.sh
# 將requirements.md文檔拆分為多個獨立文檔的腳本
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
REQUIREMENTS_FILE="docs/requirements.md"
OUTPUT_DIR="docs/requirements"
FUNCTIONAL_DIR="$OUTPUT_DIR/Functional requirements"
NONFUNCTIONAL_DIR="$OUTPUT_DIR/Non-functional requirements"

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
    mkdir -p "$FUNCTIONAL_DIR"
    mkdir -p "$NONFUNCTIONAL_DIR"
    
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
                ((++sections_count))
            fi
            
            # 開始新章節
            current_section="$section_title"
            local filename=$(clean_filename "$section_title")
            current_section_file="$OUTPUT_DIR/${filename}.md"
            echo "# $section_title" > "$current_section_file"
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
        ((++sections_count))
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

# 提取功能性需求
extract_functional_requirements() {
    log_info "提取功能性需求..."
    log_info "調試：extract_functional_requirements函數被調用"
    
    local in_functional_section=false
    local current_requirement=""
    local requirement_content=""
    local requirement_title=""
    local requirement_id=""
    local functional_count=0
    
    while IFS= read -r line; do
        # 檢查是否進入功能性需求章節
        if [[ $line =~ ^##[[:space:]]+功能性需求 ]]; then
            in_functional_section=true
            continue
        fi
        
        # 檢查是否離開功能性需求章節（遇到下一個##章節）
        if [[ $in_functional_section == true && $line =~ ^##[[:space:]]+ && ! $line =~ ^##[[:space:]]+功能性需求 ]]; then
            # 保存最後一個需求
            if [[ -n "$current_requirement" ]]; then
                save_functional_requirement "$requirement_id" "$requirement_title" "$requirement_content"
                ((++functional_count))
            fi
            in_functional_section=false
            break
        fi
        
        if [[ $in_functional_section == true ]]; then
            # 檢查是否是新的功能需求（### FR-XXX: 格式）
            if [[ $line =~ ^###[[:space:]]+(FR-[0-9]+):[[:space:]]*(.+)$ ]]; then
                # 保存前一個需求
                if [[ -n "$current_requirement" ]]; then
                    save_functional_requirement "$requirement_id" "$requirement_title" "$requirement_content"
                    ((++functional_count))
                fi
                
                # 開始新需求
                requirement_id="${BASH_REMATCH[1]}"
                requirement_title="${BASH_REMATCH[2]}"
                current_requirement="$requirement_id: $requirement_title"
                requirement_content="# $requirement_id: $requirement_title\n\n"
                continue
            fi
            
            # 添加內容到當前需求
            if [[ -n "$current_requirement" ]]; then
                requirement_content+="$line\n"
            fi
        fi
    done < "$REQUIREMENTS_FILE"
    
    # 保存最後一個需求
    if [[ $in_functional_section == true && -n "$current_requirement" ]]; then
        save_functional_requirement "$requirement_id" "$requirement_title" "$requirement_content"
        ((++functional_count))
    fi
    
    log_success "功能性需求提取完成，共 $functional_count 個需求"
}

# 保存功能性需求文件
save_functional_requirement() {
    local req_id="$1"
    local req_title="$2"
    local req_content="$3"
    
    local filename=$(clean_filename "$req_id: $req_title")
    local filepath="$FUNCTIONAL_DIR/${filename}.md"
    
    echo -e "$req_content" > "$filepath"
    log_info "已保存: $filepath"
}

# 提取非功能性需求
extract_nonfunctional_requirements() {
    log_info "提取非功能性需求..."
    log_info "調試：extract_nonfunctional_requirements函數被調用"
    
    local in_nonfunctional_section=false
    local current_requirement=""
    local requirement_content=""
    local requirement_id=""
    local requirement_title=""
    local nonfunctional_count=0
    
    while IFS= read -r line; do
        # 檢查是否進入非功能性需求章節
        if [[ $line =~ ^##[[:space:]]+非功能性需求 ]]; then
            in_nonfunctional_section=true
            continue
        fi
        
        # 檢查是否離開非功能性需求章節
        if [[ $in_nonfunctional_section == true && $line =~ ^##[[:space:]]+ && ! $line =~ ^##[[:space:]]+非功能性需求 ]]; then
            # 保存最後一個需求
            if [[ -n "$current_requirement" ]]; then
                save_nonfunctional_requirement "$requirement_id" "$requirement_title" "$requirement_content"
                ((++nonfunctional_count))
            fi
            in_nonfunctional_section=false
            break
        fi
        
        if [[ $in_nonfunctional_section == true ]]; then
            # 檢查是否是新的非功能需求（### NFR-XXX-XXX: 格式）
            if [[ $line =~ ^###[[:space:]]+(NFR-[A-Z]+-[0-9]+):[[:space:]]*(.+)$ ]]; then
                # 保存前一個需求
                if [[ -n "$current_requirement" ]]; then
                    save_nonfunctional_requirement "$requirement_id" "$requirement_title" "$requirement_content"
                    ((++nonfunctional_count))
                fi
                
                # 開始新需求
                requirement_id="${BASH_REMATCH[1]}"
                requirement_title="${BASH_REMATCH[2]}"
                current_requirement="$requirement_id: $requirement_title"
                requirement_content="# $requirement_id: $requirement_title\n\n"
                continue
            fi
            
            # 添加內容到當前需求
            if [[ -n "$current_requirement" ]]; then
                requirement_content+="$line\n"
            fi
        fi
    done < "$REQUIREMENTS_FILE"
    
    # 保存最後一個需求
    if [[ $in_nonfunctional_section == true && -n "$current_requirement" ]]; then
        save_nonfunctional_requirement "$requirement_id" "$requirement_title" "$requirement_content"
        ((++nonfunctional_count))
    fi
    
    log_success "非功能性需求提取完成，共 $nonfunctional_count 個需求"
}

# 保存非功能性需求文件
save_nonfunctional_requirement() {
    local req_id="$1"
    local req_title="$2"
    local req_content="$3"
    
    local filename=$(clean_filename "$req_id: $req_title")
    local filepath="$NONFUNCTIONAL_DIR/${filename}.md"
    
    echo -e "$req_content" > "$filepath"
    log_info "已保存: $filepath"
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
    echo "- 功能性需求文件: $(find "$FUNCTIONAL_DIR" -name "*.md" | wc -l)"
    echo "- 非功能性需求文件: $(find "$NONFUNCTIONAL_DIR" -name "*.md" | wc -l)"
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
    
    # 提取各部分內容
    extract_second_level_headers
    extract_functional_requirements
    extract_nonfunctional_requirements
    
    # 生成報告
    generate_report
    
    log_success "需求文檔拆分完成！"
}

# 執行主函數
main "$@"