"""Design-renderer tests; not acceptance tests of the proposed product."""
import copy
from html.parser import HTMLParser
import json
from pathlib import Path
import unittest
import xml.etree.ElementTree as ET

from build import HERE, render, svg, validate


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
        self.assertEqual(len(self.data['use_cases']), 16)
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
        self.assertEqual(parser.scripts, 0)
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


if __name__ == '__main__':
    unittest.main()
