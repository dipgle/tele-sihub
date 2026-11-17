"""Kiểm bản đồ tầng: bằng chứng còn trong mã, bản dựng không cũ, và cổng ĐỎ được.

    python3 -m unittest discover -s docs/so-do-tang -p 'test_*.py'
"""
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import build  # noqa: E402


class BanDoTang(unittest.TestCase):
    def test_moi_bang_chung_con_trong_ma_head(self):
        _, mat = build.tao()
        self.assertEqual(mat, [], f"bằng chứng mất: {mat}")

    def test_ban_dung_khop_nguon_va_head(self):
        noi_dung, _ = build.tao()
        self.assertEqual(build.OUT.read_text(), noi_dung, "index.html cũ — chạy build.py")

    def test_du_10_svg_va_co_bang_module(self):
        noi_dung, _ = build.tao()
        self.assertEqual(noi_dung.count("<svg "), 10)
        self.assertIn('id="bang-module"', noi_dung)

    # ── đối chứng ngược: phá đúng thứ cổng canh ⇒ phải ĐỎ ──
    def test_doi_chung_ma_mat_mau_thi_do(self):
        that = build.doc_head

        def doc_hong(tep):
            t = that(tep)
            return t.replace("r.name == a && r.is_codex()", "/* gỡ */") if tep.endswith("pipeline.rs") else t

        _, mat = build.tao(doc=doc_hong)
        self.assertIn("route_codex", mat)

    def test_doi_chung_ma_lanh_thi_khong_do(self):
        bang = {"x": {"tep": "a.rs", "mau": "foo()", "noi": "n"}}
        ra, mat = build.giai_bang_chung(bang, doc=lambda t: "a\nfoo()\nb\n")
        self.assertEqual(mat, [])
        self.assertEqual(ra["x"]["dong"], [2])

    def test_dem_module_dem_dung_nhac_va_re(self):
        lop = {"giao_tiep": ["pipeline"], "quan_ly_model": []}
        ma = "fn a() {}\nif acc.is_codex() { claude }\nlet x = 1;\n// opencode\n"
        mods = build.dem_module(lop, doc=lambda t: ma, tep_list=["rust/src/pipeline.rs"])
        self.assertEqual(mods[0]["nhac"], 2)
        self.assertEqual(mods[0]["re"], 1)
        self.assertEqual(mods[0]["lop"], "gt")

    def test_khong_doc_duoc_ma_la_ma_2_khong_phai_xanh(self):
        goc = build.tao

        def hong(doc=None):
            raise build.KhongDoDuoc("giả lập")

        build.tao = hong
        try:
            self.assertEqual(build.main([]), 2)
        finally:
            build.tao = goc


if __name__ == "__main__":
    unittest.main()
