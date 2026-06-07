# 測試指南：滑鼠點擊顯示 Process 資訊

## 修改摘要

已實現滑鼠點擊 process 時自動顯示該 process 的詳細資訊：

1. **主畫面 Process 面板**：點擊任何 process → 打開 modal 顯示詳細資訊
2. **Detail Popup (按 3)**：點擊任何 process → 打開嵌套 modal 顯示詳細資訊

## 功能詳情

### 主畫面 Process 面板（原有功能增強）

**之前的行為**：
- 滑鼠點擊只會選中 process（高亮顯示）
- 必須再按 Enter 才能看到詳細資訊

**現在的行為**：
- 滑鼠點擊會選中 process **並且**立即打開 modal 顯示詳細資訊
- 等同於：點擊 = 選中 + 按 Enter

**實現位置**：`src/main.rs` line 1047-1054

### Detail Popup 中的 Process 列表（新功能）

**之前的行為**：
- 按 3 打開 Process popup 後，滑鼠點擊完全無效
- 必須用 ↑/↓ 鍵選擇，然後按 Enter 打開嵌套 modal

**現在的行為**：
- 滑鼠點擊會選中 process **並且**立即打開嵌套 popup modal
- 等同於：點擊 = ↓ 選中 + 按 Enter

**實現位置**：`src/main.rs` line 1000-1057

## 測試場景

### 測試 1: 主畫面 Process 面板滑鼠點擊

```
步驟：
1. 執行：cargo run --release
2. 確保在 Process 面板（或用 Tab 切換到 Process）
3. 用滑鼠點擊任意一個 process
   ✅ 預期：立即打開 modal 顯示該 process 的詳細資訊
   ✅ 預期：modal 顯示的 PID、名稱等資訊與點擊的 process 一致
4. 按 'q' 關閉 modal
5. 點擊另一個 process
   ✅ 預期：打開該 process 的 modal
6. 按 'q' 關閉 modal
```

### 測試 2: Detail Popup 滑鼠點擊（新功能）

```
步驟：
1. 執行：cargo run --release
2. 按 '3' 打開 Process detail popup
3. 用滑鼠點擊 popup 中的任意一個 process
   ✅ 預期：該 process 被選中（黑底青字高亮）
   ✅ 預期：立即打開嵌套 popup modal 顯示該 process 的詳細資訊
   ✅ 預期：modal 顯示的資訊與點擊的 process 一致
4. 按 'q' 關閉 popup modal（回到 popup）
   ✅ 預期：選中狀態保持在剛才點擊的 process
5. 點擊另一個 process
   ✅ 預期：打開該 process 的 popup modal
6. 按 'q' 兩次退出到主畫面
```

### 測試 3: 排序後的滑鼠點擊準確性

```
步驟：
1. 執行：cargo run --release
2. 按 '3' 打開 Process popup
3. 按 's' 切換排序字段（例如切換到 Memory 排序）
4. 等待 process 列表重新排序
5. 點擊第一個 process（通常是記憶體使用最高的）
   ✅ 預期：打開的 modal 顯示該 process 的正確資訊
   ✅ 預期：不會顯示其他 process 的資訊
6. 按 'q' 關閉 modal
7. 按 'r' 反轉排序順序
8. 點擊第一個 process（現在應該是記憶體使用最低的）
   ✅ 預期：打開的 modal 顯示正確的 process 資訊
```

### 測試 4: 滾動後的滑鼠點擊

```
步驟：
1. 執行：cargo run --release
2. 按 '3' 打開 Process popup
3. 按 'j' 或 PageDown 向下滾動幾行
4. 用滑鼠點擊可見區域中的任意 process
   ✅ 預期：打開正確的 process modal
   ✅ 預期：不會打開錯誤的 process（例如滾動前同一位置的 process）
5. 按 'q' 關閉 modal
```

### 測試 5: 滑鼠點擊精確度

```
步驟：
1. 執行：cargo run --release
2. 按 '3' 打開 Process popup
3. 仔細觀察滑鼠游標箭頭尖端
4. 將游標箭頭尖端對準某個 process 的名稱，然後點擊
   ✅ 預期：打開該 process 的 modal（不是上一行或下一行）
5. 測試多個不同位置的 process
   ✅ 預期：點擊哪個就打開哪個
```

### 測試 6: 組合測試（鍵盤 + 滑鼠）

```
步驟：
1. 執行：cargo run --release
2. 按 '3' 打開 Process popup
3. 用 ↓ 鍵選擇某個 process
4. 按 Enter 打開 modal
5. 按 'q' 關閉 modal
6. 用滑鼠點擊另一個 process
   ✅ 預期：打開該 process 的 modal
7. 按 'q' 關閉 modal
8. 用 ↓ 鍵選擇第三個 process
9. 按 Enter
   ✅ 預期：打開該 process 的 modal
```

## 技術實現細節

### 主畫面 Process 面板

**修改的代碼**（`src/main.rs` line 1047-1054）：

```rust
if process_index < processes.len() {
    process_widget.set_selected_index(process_index);
    process_widget.adjust_scroll(rect.height.saturating_sub(1) as usize);
    state.active_panel = ViewMode::Processes;
    // Open modal to show process details (like pressing Enter)
    state.selected_process_index = Some(process_index);
    state.toggle_modal();
}
```

**新增的行為**：
- 設置 `selected_process_index`
- 調用 `toggle_modal()` 打開 modal

### Detail Popup Process 列表

**新增的代碼**（`src/main.rs` line 1000-1057）：

```rust
// Priority 1: Check if detail popup is open and handle clicks within it
if state.is_detail_popup_open() && state.detail_popup_type == ui::DetailPopupType::Process {
    if let Some(popup_state) = &mut state.detail_popup {
        // Calculate popup area
        let popup_width = ((area_width as f32 * 0.8).min(120.0) as u16).max(60);
        let popup_height = ((area_height as f32 * 0.8).min(40.0) as u16).max(20);
        
        // Check if click is within popup bounds
        if click_x >= popup_x && click_x < popup_x + popup_width
            && click_y >= popup_y && click_y < popup_y + popup_height {
            
            // Calculate clicked process index
            let clicked_row = ...;
            let process_index = popup_state.scroll_offset + clicked_row;
            
            // Select and open nested modal
            popup_state.selected_index = Some(process_index);
            let pid = processes[process_index].pid;
            state.open_popup_modal(pid);
        }
    }
}
```

**邏輯流程**：
1. 檢查是否在 Process detail popup 中
2. 計算 popup 區域（與渲染時一致）
3. 檢查點擊是否在 popup 內部
4. 計算點擊的行數（考慮滾動偏移）
5. 設置選中索引並打開嵌套 modal

### 座標計算

**Popup 區域計算**：
- 寬度：終端寬度的 80%（最大 120，最小 60）
- 高度：終端高度的 80%（最大 40，最小 20）
- X 位置：居中
- Y 位置：居中

**內容區域計算**：
- `content_start_y = popup_y + 3`（邊框1 + 標題1 + 表頭1）
- `clicked_row = (adjusted_y - content_start_y)`
- `process_index = scroll_offset + clicked_row`

**滑鼠游標調整**：
- 主面板：`adjusted_y = click_y - 1`
- Popup：`adjusted_y = click_y - 1`
- 原因：滑鼠游標的觸發點在中心而非箭頭尖端

## 編譯狀態

✅ 編譯成功：0 errors, 48 warnings

## 預期結果

所有 6 個測試場景都應該通過：
1. ✅ 主畫面點擊打開 modal
2. ✅ Detail popup 點擊打開嵌套 modal
3. ✅ 排序後點擊準確
4. ✅ 滾動後點擊準確
5. ✅ 滑鼠游標精確度
6. ✅ 鍵盤和滑鼠混合使用

## 與之前功能的一致性

- ✅ PID 追蹤：使用 PID 而非 index，即使 process 列表重新排序也能正確顯示
- ✅ 滑鼠精確度：已修正游標中心 vs 箭頭尖端的問題
- ✅ 選中狀態：滑鼠點擊會正確設置選中狀態（高亮顯示）
