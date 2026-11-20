//! Nhãn phiên KHÔNG được suy từ đường dẫn nằm trong log / kết quả công cụ.
//!
//! 🔴 Ca thật 16/11 (`docs/ra-soat-ten-phien-goc-20261116.md`): phiên acc9 mở ở GỐC
//! workspace, chủ máy chỉ hỏi "kiểm tra máy bị tắt đột ngột" — chưa giao dự án
//! nào. Một kết quả công cụ có chứa đường dẫn `zkey/…` hai lần, và nhãn đổi
//! `[projects]` → `[zkey]` → `[scripts]`. Phép đếm đường dẫn trả lời *"phiên đụng
//! vào thư mục nào"*, không phải *"phiên được giao dự án nào"*.
//!
//! Ba thứ tách riêng (TODO.md, nhãn phiên gốc):
//! - **scope** = thư mục CLI mở phiên (`cwd`) — CLI tự khai, không đếm;
//! - **chủ đề** = thứ log nhắc tới — KHÔNG bao giờ đổi danh tính;
//! - **danh tính** = lời phiên tự khai `[dự-án]` — một lần là NHẮC, hai lần mới
//!   là KHAI (trừ khi khớp với scope).

/// Gốc workspace dựng tạm: `huba`, `zkey`, `scripts` là dự án thật (có `CLAUDE.md`),
/// nên không phụ thuộc máy nào đang chạy bài kiểm.
struct Ws {
    _tmp: tempfile::TempDir,
    root: String,
}

fn ws() -> Ws {
    let tmp = tempfile::tempdir().expect("thư mục tạm");
    for p in ["huba", "zkey", "scripts"] {
        let d = tmp.path().join(p);
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(d.join("CLAUDE.md"), "# x").unwrap();
    }
    let root = tmp.path().to_string_lossy().to_string();
    Ws { _tmp: tmp, root }
}

fn lines(v: Vec<serde_json::Value>) -> String {
    v.iter()
        .map(|x| x.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Một lượt nói của phiên.
fn noi(text: &str) -> serde_json::Value {
    serde_json::json!({
        "type": "assistant",
        "message": {"role": "assistant", "content": [{"type": "text", "text": text}]}
    })
}

/// Một kết quả công cụ nhắc tới đường dẫn của dự án `p` — đúng hình dạng đã gây lỗi.
fn cong_cu_nhac(root: &str, p: &str) -> serde_json::Value {
    serde_json::json!({
        "type": "user", "cwd": root,
        "message": {"role": "user", "content": [{
            "type": "tool_result",
            "content": format!("{root}/{p}/native/src/main.rs {root}/{p}/Cargo.toml")
        }]}
    })
}

/// CA GỐC: không ai giao dự án, log nhắc `zkey` nhiều lần ⇒ KHÔNG có dự án.
#[test]
fn duong_dan_trong_log_khong_tu_dat_ten_du_an() {
    let w = ws();
    let tail = lines(vec![
        noi("Đang kiểm tra máy bị tắt đột ngột."),
        cong_cu_nhac(&w.root, "zkey"),
        cong_cu_nhac(&w.root, "zkey"),
        cong_cu_nhac(&w.root, "zkey"),
        cong_cu_nhac(&w.root, "scripts"),
    ]);
    assert_eq!(
        huba::sessions::folder_for_session("t-log-zkey", &tail, &w.root, ""),
        None,
        "đường dẫn nằm trong kết quả công cụ đã đặt tên dự án cho một phiên chưa được giao"
    );
}

/// Không lời khai nào, cũng không đường dẫn nào ⇒ vẫn không bịa.
#[test]
fn khong_bang_chung_thi_khong_co_du_an() {
    let w = ws();
    let tail = lines(vec![noi("Xin chào, tôi giúp được gì?")]);
    assert_eq!(
        huba::sessions::folder_for_session("t-trong", &tail, &w.root, ""),
        None
    );
}

/// Chỉ MỘT lượt nói mở đầu bằng `[huba]` — assistant tự viết, chưa phải bằng
/// chứng độc lập về ý định chủ máy ⇒ không đủ để GIEO tên.
#[test]
fn mot_tien_to_assistant_khong_du_de_gieo_ten() {
    let w = ws();
    let tail = lines(vec![noi("[huba] Lượt nói đầu tiên của phiên này.")]);
    assert_eq!(
        huba::sessions::folder_for_session("t-mot-lan", &tail, &w.root, ""),
        None,
        "một lần khai đã gieo được tên"
    );
}

/// Chiều ngược: HAI lượt nói nhất quán là khai thật ⇒ gieo được.
#[test]
fn hai_tien_to_nhat_quan_thi_gieo_duoc_ten() {
    let w = ws();
    let tail = lines(vec![
        noi("[huba] Đang đọc quota.rs."),
        noi("[huba] Đã vá xong, chạy bài kiểm."),
    ]);
    assert_eq!(
        huba::sessions::folder_for_session("t-hai-lan", &tail, &w.root, "").as_deref(),
        Some("huba")
    );
}

/// Đã khai đủ rồi thì đường dẫn tràn ngập log cũng không lật được (giữ luật 18/08).
#[test]
fn da_khai_thi_log_nhac_du_an_khac_khong_lat_duoc() {
    let w = ws();
    let sid = "t-giu-khai";
    let khai = lines(vec![noi("[huba] Việc một."), noi("[huba] Việc hai.")]);
    assert_eq!(
        huba::sessions::folder_for_session(sid, &khai, &w.root, "").as_deref(),
        Some("huba")
    );
    let sau = lines(vec![
        cong_cu_nhac(&w.root, "zkey"),
        cong_cu_nhac(&w.root, "zkey"),
        cong_cu_nhac(&w.root, "zkey"),
    ]);
    assert_eq!(
        huba::sessions::folder_for_session(sid, &sau, &w.root, "").as_deref(),
        Some("huba")
    );
}

// ───────────── scope = cwd: dữ kiện CLI tự khai, không phải thứ đếm trong log ─────────────

/// Phiên MỞ TRONG `huba` mà không khai gì, log nhắc `zkey` đầy ⇒ vẫn là `huba`.
/// (Bản đếm cũ cho kết quả này nhờ `cwd` nằm trong mọi bản ghi; bây giờ nó đến
/// từ chính scope — nên bỏ phép đếm không làm mất nhãn của phiên mở trong dự án.)
#[test]
fn phien_mo_trong_du_an_giu_nhan_scope_du_log_nhac_du_an_khac() {
    let w = ws();
    let tail = lines(vec![
        cong_cu_nhac(&w.root, "zkey"),
        cong_cu_nhac(&w.root, "zkey"),
        cong_cu_nhac(&w.root, "zkey"),
    ]);
    assert_eq!(
        huba::sessions::folder_for_session("t-scope-log", &tail, &w.root, "huba").as_deref(),
        Some("huba")
    );
}

/// Một lượt NHẮC `zkey` trong phiên mở trong `huba` không lật được scope.
#[test]
fn mot_lan_nhac_du_an_khac_khong_lat_scope() {
    let w = ws();
    let tail = lines(vec![noi("[zkey] Đang đọc native/src/main.rs.")]);
    assert_eq!(
        huba::sessions::folder_for_session("t-scope-nhac", &tail, &w.root, "huba").as_deref(),
        Some("huba")
    );
    // Và làn của lời nhắc bị từ chối KHÔNG được gắn vào dự án đang giữ.
    assert_eq!(
        huba::sessions::lane_for_session(
            "t-scope-nhac",
            &lines(vec![noi("[zkey/native] Đang đọc.")]),
            &w.root,
            || None
        ),
        "",
        "làn của một lời nhắc bị từ chối đã bị gắn vào scope"
    );
}

/// Hai lượt khai nhất quán `zkey` là KHAI thật: đổi được dù scope là `huba`.
#[test]
fn hai_lan_khai_du_an_khac_van_doi_duoc_khoi_scope() {
    let w = ws();
    let tail = lines(vec![
        noi("[zkey] Chuyển sang zkey."),
        noi("[zkey] Đang vá."),
    ]);
    assert_eq!(
        huba::sessions::folder_for_session("t-scope-doi", &tail, &w.root, "huba").as_deref(),
        Some("zkey")
    );
}

/// Một lượt khai khớp scope kèm làn ⇒ gieo ngay, làn đi theo.
#[test]
fn mot_lan_khai_khop_scope_mang_theo_lan() {
    let w = ws();
    let tail = lines(vec![noi("[huba/ui] Đang kiểm bản thiết kế.")]);
    assert_eq!(
        huba::sessions::folder_for_session("t-scope-lan", &tail, &w.root, "huba").as_deref(),
        Some("huba")
    );
    assert_eq!(
        huba::sessions::lane_for_session("t-scope-lan", &tail, &w.root, || None),
        "ui"
    );
}

/// Làn không tự dựng nhãn dự án: phiên mở ở gốc + MỘT lượt khai chưa được nhận
/// thì chỗ hiển thị không ghép làn vào đâu cả.
#[test]
fn lan_khong_dung_mot_minh_lam_nhan_du_an() {
    let w = ws();
    let tail = lines(vec![noi("[huba/ui] Đang kiểm.")]);
    assert_eq!(
        huba::sessions::folder_for_session("t-chua-nhan", &tail, &w.root, ""),
        None
    );
    // `lane_for_session` đọc đuôi trực tiếp; không có dự án nào được nhận thì nó
    // vẫn có thể trả làn của lời khai đơn — ghi nhận hành vi này để không ai tin
    // làn là danh tính: chỗ hiển thị chỉ ghép làn khi `folder` không rỗng.
    // (`label_sessions` dùng `s.name` khi folder rỗng, bỏ qua làn.)
    let mut rows = vec![huba::sessions::LiveSession {
        session_id: "t-chua-nhan-0000-0000-0000-000000000000".into(),
        name: "Kiểm tra máy".into(),
        folder: String::new(),
        lane: "ui".into(),
        ..Default::default()
    }];
    huba::sessions::label_sessions(&mut rows, std::path::Path::new(&w.root));
    assert_eq!(
        rows[0].label, "Kiểm tra máy",
        "làn không tự dựng nhãn dự án"
    );
}

/// `scope_label`: dự án của thư mục CLI mở phiên, đo trên đĩa.
#[test]
fn scope_label_doc_thu_muc_mo_phien() {
    let w = ws();
    let d = |p: &str| format!("{}/{p}", w.root);
    // Gốc workspace ⇒ chưa giao dự án.
    assert_eq!(huba::sessions::scope_label(&w.root, &w.root), "");
    assert_eq!(
        huba::sessions::scope_label(&w.root, &format!("{}/", w.root)),
        ""
    );
    // Ngoài workspace ⇒ không scope.
    assert_eq!(huba::sessions::scope_label(&w.root, "/tmp"), "");
    assert_eq!(huba::sessions::scope_label(&w.root, ""), "");
    // Trong dự án (có marker) ⇒ dừng ở dự án, kể cả thư mục con.
    assert_eq!(huba::sessions::scope_label(&w.root, &d("huba")), "huba");
    assert_eq!(
        huba::sessions::scope_label(&w.root, &d("huba/rust/src")),
        "huba"
    );
    // Ngăn kéo (không marker) ⇒ mở ra dự án bên trong, như `open_drawer`.
    std::fs::create_dir_all(d("AI/tcc/amm")).unwrap();
    std::fs::write(d("AI/tcc/amm/CLAUDE.md"), "# x").unwrap();
    assert_eq!(
        huba::sessions::scope_label(&w.root, &d("AI/tcc/amm/src")),
        "AI/tcc/amm"
    );
    assert_eq!(huba::sessions::scope_label(&w.root, &d("AI/tcc")), "AI/tcc");
    // Không đo được (thư mục không tồn tại) ⇒ giữ bậc đầu, đừng đoán sâu thêm.
    assert_eq!(
        huba::sessions::scope_label(&w.root, &d("khong-co/a/b")),
        "khong-co"
    );
}
