# Phase 3 測試指南

## 建置並執行

```bash
# 建置專案
cargo build --release

# 執行應用程式
cargo run --release
```

## 預期畫面

應該會看到一個全螢幕的終端介面，顯示：

### CPU 面板
- 頂部：CPU 使用率進度條（彩色，根據使用率變色）
- 詳細資訊：
  - User time (藍綠色)
  - System time (黃色)
  - Idle time (綠色)
  - IOWait time (紫色)

### Memory 面板
- 頂部：記憶體使用率進度條（彩色漸層）
- 詳細資訊：
  - Available memory (綠色)
  - Cached memory (藍綠色)
  - Buffers (黃色)
  - Swap usage (紫色，如果有設定 swap)

### 進階功能（已實作）
您的版本還包含了 Phase 4+ 的功能：
- **Process 面板**：顯示執行中的程序列表
- **多視圖切換**：CPU、Memory、Processes
- **互動式導航**：上下捲動、排序、篩選

## 鍵盤控制

### 基本控制
- `q` 或 `Esc` - 離開程式
- `?` 或 `h` - 顯示/隱藏說明畫面

### 視圖切換
- `Tab` - 下一個視圖
- `Shift+Tab` - 上一個視圖
- `1` - 切換到 CPU 視圖
- `2` - 切換到 Memory 視圖
- `3` - 切換到 Processes 視圖

### Process 視圖控制（Phase 4+）
- `j` / `↓` - 向下捲動
- `k` / `↑` - 向上捲動
- `g` - 跳到頂部
- `G` - 跳到底部
- `Ctrl+D` - 向下翻頁
- `Ctrl+U` - 向上翻頁
- `/` - 開啟篩選功能
- `s` - 循環排序欄位
- `r` - 反轉排序順序
- `Enter` - 顯示程序詳細資訊

## 驗證重點

### Phase 3 核心功能
✅ TUI 正常初始化並佔滿整個終端
✅ CPU 和 Memory 面板正確顯示
✅ 數據每秒更新（預設 1 秒間隔）
✅ 進度條顏色根據使用率變化（低用量=藍/綠，高用量=黃/紅）
✅ 按 `q` 可以乾淨地退出程式
✅ 終端狀態在退出後正確還原

## 故障排除

### 如果遇到錯誤：

1. **"No such device or address"**
   - 確保在真正的終端中執行（不是背景執行）
   - 不要使用 `&` 或 `nohup`

2. **畫面顯示異常**
   - 確保終端視窗夠大（至少 80x24）
   - 檢查終端支援 ANSI 色彩

3. **編譯錯誤**
   ```bash
   # 清理並重新建置
   cargo clean
   cargo build
   ```

## 效能指標

Phase 3 應該達到：
- CPU 使用率 < 5% （監控工具本身）
- 記憶體使用 < 50MB
- 畫面更新流暢（60 FPS）
- 無明顯延遲或卡頓
