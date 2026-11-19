"""Design-renderer tests; not acceptance tests of the proposed product."""
import copy
from html.parser import HTMLParser
import json
from pathlib import Path
import unittest
import xml.etree.ElementTree as ET

from build import HERE, mermaid, render, svg, validate
from workspace_mockup import SESSIONS, standalone
from unittest.mock import patch


class PageParser(HTMLParser):
    def __init__(self):
        super().__init__()
        self.ids = []
        self.links = []
        self.remote = []
        self.scripts = 0

    def handle_starttag(self, tag, attrs):
        data = dict(attrs)
        if 'id' in data:
            self.ids.append(data['id'])
        if tag == 'a' and data.get('href', '').startswith('#'):
            self.links.append(data['href'][1:])
        for key in ('src', 'href'):
            if data.get(key, '').startswith(('https:', 'http:', '//')):
                self.remote.append(data[key])
        self.scripts += tag == 'script'


class RenderTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.data = json.loads((HERE / 'design.json').read_text(encoding='utf-8'))

    def test_links_uc_layers_and_all_participants(self):
        validate(self.data)
        self.assertEqual(len(self.data['use_cases']), 22)
        self.assertEqual(len(self.data['flows']), 19)
        self.assertEqual(len(self.data['front_tables'][0]['rows']), 30)
        components = self.data['front_tables'][0]['rows']
        self.assertEqual([r[0] for r in components], [f'A{i:02}' for i in range(1,31)])
        self.assertTrue(all(len(r) == 4 and all(r) for r in components))
        self.assertIn('X0', {f['id'] for f in self.data['flows']})

    def test_every_svg_is_well_formed_and_labelled(self):
        ns = {'s': 'http://www.w3.org/2000/svg'}
        for flow in self.data['flows']:
            root = ET.fromstring(svg(flow))
            self.assertIsNotNone(root.find('s:title', ns))
            self.assertIsNotNone(root.find('s:desc', ns))
            arrows = root.findall('.//s:line[@marker-end]', ns)
            self.assertEqual(len(arrows), sum(e[0] != e[1] for e in flow['events']))

    def test_offline_page_unique_ids_and_local_navigation(self):
        md, page = render(self.data)
        parser = PageParser()
        parser.feed(page)
        self.assertEqual(len(parser.ids), len(set(parser.ids)))
        self.assertGreater(len(parser.links), 10)
        self.assertTrue(set(parser.links).issubset(set(parser.ids)))
        self.assertEqual(parser.remote, [])
        self.assertEqual(parser.scripts, 2)  # fixture JSON + scoped prototype behavior
        self.assertEqual(md.count('```mermaid'), len(self.data['flows']))

    def test_generated_documents_match_current_source(self):
        md, page = render(self.data)
        self.assertEqual((HERE / 'index.md').read_text(encoding='utf-8'), md)
        self.assertEqual((HERE / 'index.html').read_text(encoding='utf-8'), page)

    def test_negative_unknown_participant_rejected(self):
        broken = copy.deepcopy(self.data)
        broken['flows'][0]['events'][0][1] = 99
        with self.assertRaisesRegex(ValueError, 'Unknown participant'):
            validate(broken)
        validate(self.data)

    def test_negative_missing_flow_rejected(self):
        broken = copy.deepcopy(self.data)
        broken['use_cases'][0][4] = ['NOT-A-FLOW']
        with self.assertRaisesRegex(ValueError, 'link existing flows'):
            validate(broken)
        validate(self.data)

    def test_negative_missing_layer_and_duplicate_id_rejected(self):
        broken = copy.deepcopy(self.data)
        broken['flows'][1]['id'] = broken['flows'][0]['id']
        with self.assertRaisesRegex(ValueError, 'Duplicate flow'):
            validate(broken)
        broken = copy.deepcopy(self.data)
        for flow in broken['flows']:
            flow['layer'] = 'UC'
        with self.assertRaisesRegex(ValueError, 'All three layers'):
            validate(broken)

    def test_negative_long_label_not_silently_clipped(self):
        flow = copy.deepcopy(self.data['flows'][1])
        flow['events'][0] = [0, 0, 'label ' * 100, 'negative readability control', 'normal']
        with self.assertRaisesRegex(ValueError, 'label too long'):
            svg(flow)

    def test_labels_are_data_not_executable_markup(self):
        flow = copy.deepcopy(self.data['flows'][1])
        flow['events'][0][2] = '<script>oops()</script>'
        rendered = svg(flow)
        self.assertNotIn('<script>', rendered)
        self.assertIn('&lt;script&gt;', rendered)
        ET.fromstring(rendered)

    def test_issue_census_and_marks_at_real_steps(self):
        issues = self.data['issues']
        self.assertEqual([i['id'] for i in issues], [f'P{i:02}' for i in range(1, 23)])
        self.assertEqual({i['state'] for i in issues}, {'bug', 'gap', 'decision', 'risk'})
        for flow in self.data['flows']:
            root = ET.fromstring(svg(flow, issues))
            ns = {'s': 'http://www.w3.org/2000/svg'}
            actual = [a.get('href') for a in root.findall('.//s:a', ns)]
            expected = ['#' + i['id'] for step in range(1, len(flow['events']) + 1)
                        for i in issues if [flow['id'], step] in i['points']]
            self.assertEqual(actual, expected)

    def test_negative_issue_guards_both_directions(self):
        for field, value, error in (
            ('state', 'resolved-without-evidence', 'Unknown issue state'),
            ('points', [['MISSING', 1]], 'unknown flow or step'),
            ('points', [['U1', 999]], 'unknown flow or step'),
            ('evidence', '', 'requires evidence'),
        ):
            broken = copy.deepcopy(self.data)
            broken['issues'][0][field] = value
            with self.assertRaisesRegex(ValueError, error):
                validate(broken)
            validate(self.data)
        broken = copy.deepcopy(self.data)
        broken['issues'].append(copy.deepcopy(broken['issues'][0]))
        with self.assertRaisesRegex(ValueError, 'Duplicate issue ID'):
            validate(broken)

    def test_issue_markup_is_escaped(self):
        data = copy.deepcopy(self.data)
        data['issues'][0]['evidence'] = '<script>bad()</script>'
        _, page = render(data)
        self.assertNotIn('<script>bad()', page)
        self.assertIn('&lt;script&gt;', page)

    def test_v2_overview_is_explicitly_a_design(self):
        _, page = render(self.data)
        self.assertIn('A∩B', page)
        self.assertIn('không cần mở thường xuyên', page)
        self.assertIn('Không phải app đã chạy', page)
        self.assertIn('Phiên chung', page)
        self.assertIn('Đóng panel', page)
        self.assertIn('Terminal nhúng', page)

    def test_error_branch_breaks_instead_of_falling_through(self):
        for flow in self.data['flows']:
            diagram = mermaid(flow)
            groups = sum(e[4] == 'error' and (i == 0 or flow['events'][i-1][4] != 'error')
                         for i, e in enumerate(flow['events']))
            self.assertEqual(diagram.count('    break '), groups)
            for event in flow['events']:
                if event[4] == 'error':
                    self.assertNotIn('    opt ' + event[3] + '\n', diagram)

    def test_source_failure_still_reports_unknown_before_break_ends(self):
        flow = next(f for f in self.data['flows'] if f['id'] == 'D5')
        diagram = mermaid(flow)
        self.assertEqual(diagram.count('    break '), 1)
        self.assertIn('    alt ', diagram)
        error_path = diagram.split('    break ', 1)[1].split('    end', 1)[0]
        self.assertIn('Probe lỗi / disconnect / schema chưa biết', error_path)
        self.assertIn('Cũ/unknown và mốc cuối đã xác minh', error_path)

    def test_standalone_workspace_matches_source_and_is_labelled_demo(self):
        page = standalone()
        self.assertEqual((HERE / 'workspace.html').read_text(), page)
        self.assertIn('B luôn hiện ở đáy khung làm việc', page)
        self.assertIn('chưa kết nối CLI', page)
        self.assertIn('data-workspace-script', page)
        self.assertNotIn('class="wire-list"', page)
        self.assertEqual(len(SESSIONS), 7)
        self.assertEqual(len({s['id'] for s in SESSIONS}), 7)

    def test_fixture_cannot_close_script_and_inject_another(self):
        sessions = copy.deepcopy(SESSIONS)
        sessions[0]['latest'] = '</script><script>alert(1)</script>'
        with patch('workspace_mockup.SESSIONS', sessions):
            page = standalone()
        parser = PageParser()
        parser.feed(page)
        self.assertEqual(parser.scripts, 2)
        self.assertIn('<\\/script>', page)


if __name__ == '__main__':
    unittest.main()
