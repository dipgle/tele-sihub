# Theo dõi một yêu cầu từ model → tool → model → kết thúc lượt

15/11/2026 20:51 +07. **Đề xuất để Hà chọn A/B; chưa triển khai.** Bàn giao: `.tmp/chuyen-ui-theo-doi-vong-model-20261115.md:3–26`. Bản vẽ nền: `index.html`, đặc biệt X0/C2.

## 1. Có thể quan sát gì?

```text
Yêu cầu được CLI nhận
  └─ Vòng model 1
      ├─ text/reasoning nếu nguồn xuất
      ├─ tool-call ID, tên, input, chờ quyền/chạy/kết quả/lỗi
      └─ tool result được CLI xử lý
          └─ Vòng model 2 … N
              └─ phản hồi cuối / lỗi / ngắt
                  └─ CLI trả quyền điều khiển: idle + outcome nếu có
```

Một vòng có thể gọi nhiều tool; nhiều tool có thể chạy song song. Subagent có nhánh/phiên con riêng khi nguồn cho biết. MCP và tool native dùng cùng khuôn hiển thị nhưng giữ nhãn đường thực thi. Tool trả kết quả **không tự chứng minh mọi byte kết quả đã được đưa vào request model kế tiếp**. Nhật ký không phải toàn bộ payload/context model.

`idle` chỉ cho biết runtime đã về trạng thái rảnh; kể cả `outcome=succeeded` cũng không tự chứng minh công việc nghiệp vụ đã nghiệm thu. UI cần nhãn **“Lượt xử lý kết thúc — CLI báo succeeded”**, không tự suy thành **“Công việc hoàn thành đúng”**.

## 2. Đối chiếu chỉ đọc trong lượt này

Nguồn: SQLite mở `-readonly`, `/home/user1/.opencode-acc9/data/opencode/opencode.db`, `session_message`, phiên `ses_e20c76d70ffe6bWCKWnR046ALP`, phạm vi `seq 4…102`. Chỉ đọc metadata/schema/count; không in nội dung yêu cầu, lệnh, kết quả tool hoặc credential. Đây là **thăm dò nguồn**, không nghiệm thu UI.

| Mẫu số / trường | Kết quả |
|---|---|
| Hàng trong phạm vi | 11: user1, assistant7, location-switched1, system1, idle1 |
| Assistant finish | tool-calls6, stop1 |
| Tool blocks | 15/15 completed: read7, shell4, execute2, glob2 |
| Tool ID | 15 có ID / 15 duy nhất / 0 thiếu |
| Input/output tool | `state.input` object; `state.content` array ở cả15 block; không mặc định tìm `state.output` |
| Yêu cầu bắt đầu | user seq4, 19:39:49.299 +07 |
| Assistant cuối | seq95 tạo19:42:02.288, cập nhật19:42:15.980, finish=stop |
| Runtime rảnh | seq102, idle/outcome=succeeded, cập nhật19:42:15.994 |
| Khoảng quan sát | 146695ms, khoảng2phút26.7giây; không gọi là thời gian inference thuần |
| session_part | Toàn bảng hiện0 hàng; nội dung mẫu nằm trong `session_message.data.content[]` |

**Sửa độ chính xác của bàn giao:** 19:42:02 là lúc tạo hàng assistant cuối, không phải bằng chứng nó đã kết thúc lúc đó. `time_updated` của hàng95 và idle cho thấy kết thúc được quan sát lúc19:42:15.

Mã hiện tại có đường preview/đuôi và trạng thái: `rust/src/opencode.rs:196–266`, `:269–328`, `:1000–1098`. Đuôi giới hạn40 hàng/6000ký tự chỉ phục vụ preview/nhãn, **không dùng làm nguồn timeline đầy đủ**. Chưa có màn request trace mới.

## 3. Hai cách hiển thị đang chờ chọn

| | A — Xem lại khi cần | B — Theo dõi trực tiếp |
|---|---|---|
| Cửa vào | Chọn một yêu cầu → Xem vòng xử lý | Mở khung Theo dõi yêu cầu đang chạy |
| Cấp đầu | Yêu cầu, mốc bắt đầu/kết thúc, trạng thái theo nguồn | Cùng nội dung, tự cập nhật theo nguồn mới |
| Các vòng | Model1…N, trạng thái, token nếu có, tool con | Vòng/tool đang chạy hoặc chờ, kết quả/lỗi đến đâu hiện đến đó |
| Mở chi tiết | Input tool, result/error, nguồn/ref, phần cắt/thiếu | Giống A; không kéo người dùng về cuối khi đang xem dòng cũ |
| Tổng kết | Số vòng quan sát được, số tool/errors, thời gian, coverage | Cùng tổng kết khi runtime kết thúc/ngắt/lỗi; mở lại lịch sử vẫn đọc được |
| Giới hạn hiện tại | Cần parser/correlation và UI; dữ liệu lịch sử đã thăm dò một mẫu OpenCode | Cần thêm bằng chứng nguồn live/độ trễ và cập nhật cùng một hàng; mẫu lịch sử chưa chứng minh live đầy đủ |

**Khuyến nghị A trước**, giữ cùng contract để có thể thêm B sau. Đây là khuyến nghị, không phải quyết định đã được Hà duyệt. Không gọi thêm model chỉ để theo dõi/tóm tắt trace mặc định.

## 4. Những điều phải giữ khi triển khai

- Nhận diện yêu cầu/lượt bằng ID nguồn khi có. Trong cùng phiên đã thấy user106 → assistant108 → user115 → assistant116 trước một mốc idle khác: không mặc định mỗi user mới đóng lượt trước; tin bổ sung/hàng đợi phải có nhãn tương quan chưa xác định nếu nguồn chưa chứng minh.
- Một hàng assistant có `time_created` khác `time_updated`: live reader phải theo cả thay đổi hàng cũ và trạng thái tool, không chỉ `seq > last_seq`. Snapshot lịch sử không khôi phục chắc chắn mọi trạng thái trung gian đã trôi qua.
- Không coi seq nhảy số là tự động mất dữ liệu. Không gộp hàng assistant trong DB với số request HTTP/attempt provider nếu nguồn không xuất ánh xạ đó; dùng nhãn “vòng assistant quan sát được”.
- Phân loại completed/error/denied/running/aborted/unknown theo nguồn; tool lỗi có thể được model xử lý tiếp. Pending tool, subagent và tác vụ shell nền chưa khép phải còn nhìn thấy.
- Nội dung dài có preview và đường xem nguồn, không cắt âm thầm rồi nhận đầy đủ. Không biến thông tin lệnh trong tool input thành nút chạy lại mặc định.
- Token theo vòng chỉ cộng sau khi xác minh semantics/cache và khử trùng snapshot. Dữ liệu nguồn thiếu phải hiện coverage/unknown.
- Lượt này chỉ đối chiếu một chu kỳ OpenCode đã khép. Claude/Codex và live transitions cần đo cùng contract; chưa công bố “cả ba CLI đã theo dõi đầy đủ”.

**Việc kế:** Hà chọn A hoặc B; sau đó mới chốt mockup/contract và phạm vi triển khai cùng phiên lõi. Không có job nền trong lượt thăm dò.
