#!/usr/bin/env python
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import datetime
import json
import os

from mozlog.formatters import base
from collections import defaultdict

class InteractiveHTMLFormatter(base.BaseFormatter):
    """My Formatter that produces a simple HTML-formatted report."""
    def __init__(self):
        self.unexpected_results = []
        self.suite_name = None
        self.start_times = {}
        self.suite_times = {"start": None,
                            "end": None}
        self.counts = defaultdict(lambda: 1)

    def suite_start(self, data):
        self.suite_times["start"] = data["time"]
        self.suite_name = data["source"]

    def suite_end(self, data):
        self.suite_times["end"] = data["time"]
        return self.generate_html()

    def test_start(self, data):
        self.start_times[data["test"]] = data["time"]

    def test_end(self, data):
        status = data["status"]

        if status == data.get("expected", status):
            self.counts[status] += 1
            return

        output_data = {
            'duration': (data["time"] - self.start_times.pop(data["test"])) / 1000.,
        }
        output_data.update(data)
        self.unexpected_results.append(output_data)

    def generate_html(self):
        output = {
            'counts': self.counts,
            'unexpected': self.unexpected_results,
        }

        template_path = os.path.join(os.path.dirname(__file__), 'interactive_html_formatter.html')
        template_file_contents = open(template_path).readlines()
        template_file_contents = "".join(template_file_contents)
        template_file_contents = template_file_contents.replace("{0}", json.dumps(output))
        return template_file_contents
