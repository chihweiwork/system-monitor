# 測試指南：統一 'q' 和 'ESC' 鍵行為

## 修改摘要

已實現層級式關閉邏輯，'q' 和 'ESC' 現在具有相同的行為：
- 優先關閉最上層的視窗/彈窗
- 如果沒有任何視窗打開，則退出程序

## 視窗關閉優先級（從高到低）

1. Help screen（幫助畫面）
2. Popup modal（嵌套的 process 詳情彈窗）
3. Detail popup（詳細彈窗）
4. Main modal（主 process 詳情彈窗）
5. Filter mode（過濾模式）
6. 退出程序

## 測試場景

### 測試 1: 基本層級關閉（Popup Modal）

```
步驟：
1. 執行：cargo run --release
2. 按 '3' 打開 Process popup
3. 按 '↓' 選擇一個 process
4. 按 'Enter' 打開 popup modal（嵌套彈窗）
5. 按 'q' → ✅ 應該關閉 popup modal（回到 popup）
6. 按 'q' → ✅ 應該關閉 popup（回到主界面）
7. 按 'q' → ✅ 應該退出程序
```

### 測試 2: ESC 與 q 行為一致

```
步驟：
1. 執行：cargo run --release
2. 按 '3' 打開 Process popup
3. 按 'ESC' → ✅ 應該關閉 popup
4. 按 'ESC' → ✅ 應該退出程序
```

### 測試 3: Help Screen

```
步驟：
1. 執行：cargo run --release
2. 按 '?' 打開 help
3. 按 'q' → ✅ 應該關閉 help
4. 按 '?' 再次打開 help
5. 按 'ESC' → ✅ 應該關閉 help
6. 按 'q' → ✅ 應該退出程序
```

### 測試 4: Filter Mode

```
步驟：
1. 執行：cargo run --release
2. 在 Process 面板（預設）按 '/' 進入 filter mode
3. 輸入一些文字（如 "fire"）
4. 按 'q' → ✅ 應該退出 filter mode（不應該輸入字符 'q'）
5. 按 '/' 再次進入 filter mode
6. 按 'ESC' → ✅ 應該退出 filter mode
7. 按 'q' → ✅ 應該退出程序
```

### 測試 5: Main Modal

```
步驟：
1. 執行：cargo run --release
2. 在 Process 面板選擇一個 process（用滑鼠點擊或用 'd' 鍵）
3. 按 'Enter' 打開 modal
4. 按 'q' → ✅ 應該關閉 modal
5. 再次打開 modal
6. 按 'ESC' → ✅ 應該關閉 modal
7. 按 'q' → ✅ 應該退出程序
```

### 測試 6: Detail Popup (非 Process)

```
步驟：
1. 執行：cargo run --release
2. 按 '1' 打開 CPU detail popup
3. 按 'q' → ✅ 應該關閉 popup
4. 按 '2' 打開 Memory detail popup
5. 按 'ESC' → ✅ 應該關閉 popup
6. 按 'q' → ✅ 應該退出程序
```

### 測試 7: Search Mode（特殊情況 - 應該允許輸入 'q'）

```
步驟：
1. 執行：cargo run --release
2. 按 '3' 打開 Process popup
3. 按 '/' 進入 search mode
4. 按 'q' → ✅ 應該輸入字符 'q'（不應該關閉）
5. 顯示應該是 "Search: q"
6. 按 'ESC' → ✅ 應該退出 search mode（清空搜尋）
7. 按 'q' → ✅ 應該關閉 popup
8. 按 'q' → ✅ 應該退出程序
```

### 測試 8: 組合場景（多層視窗）

```
步驟：
1. 執行：cargo run --release
2. 按 '/' 進入 filter mode
3. 按 'q' → ✅ 應該退出 filter mode（不退出程序）
4. 按 '3' 打開 Process popup
5. 按 '/' 進入 popup search mode
6. 按 'ESC' → ✅ 應該退出 search mode
7. 按 '?' 打開 help（help 優先級最高）
8. 按 'q' → ✅ 應該關閉 help（不應該關閉 popup）
9. 按 'q' → ✅ 應該關閉 popup
10. 按 'q' → ✅ 應該退出程序
```

## 技術實現細節

### 修改的文件

1. **src/ui/state.rs**
   - 新增 `handle_close_key()` 方法，實現層級式關閉邏輯
   - 返回 `true` 表示關閉了一個視窗，`false` 表示應該退出程序

2. **src/main.rs**
   - 主事件循環：統一 'q' 和 'ESC' 的處理，都使用 `handle_close_key()`
   - `handle_process_view_input`：更新 filter mode，使 'q' 可以關閉 filter
   - Detail popup 和 Popup modal 的處理已經正確

### 邏輯流程

```rust
主事件循環中的 'q' 或 'ESC'：
  if !filter_active {
    if !handle_close_key() {
      break;  // 退出程序
    }
  }

handle_close_key() 的邏輯：
  if help_visible → 關閉 help, return true
  else if popup_modal_active → 關閉 popup modal, return true
  else if detail_popup_open → 關閉 detail popup, return true
  else if modal_active → 關閉 modal, return true
  else if filter_active → 關閉 filter, return true
  else → return false (應該退出程序)
```

## 編譯狀態

✅ 編譯成功：0 errors, 48 warnings

## 特殊情況處理

1. **Search mode in popup**: 'q' 被當作普通字符輸入（正確）
2. **Filter mode**: 'q' 關閉 filter mode（正確）
3. **Modal priority**: Popup modal > Detail popup > Main modal（正確）

## 預期結果

所有 8 個測試場景都應該通過，'q' 和 'ESC' 的行為完全一致。
