#!/usr/bin/env python

# Copyright 2023 The Servo Project Developers.
#
# Permission is hereby granted, free of charge, to any person obtaining a copy
# of this software and associated documentation files (the "Software"), to deal
# in the Software without restriction, including without limitation the rights
# to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
# copies of the Software, and to permit persons to whom the Software is
# furnished to do so, subject to the following conditions:
#
# The above copyright notice and this permission notice shall be included in all
# copies or substantial portions of the Software.
#
# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
# IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
# FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
# AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
# LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
# OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
# SOFTWARE.

# pylint: disable=missing-docstring
# pylint: disable=invalid-name

import json
import logging
import os
import sys

from wptupstreamer import WPTSync

def main() -> int:
    context = json.loads(os.environ['GITHUB_CONTEXT'])
    logging.getLogger().level = logging.INFO

    success = WPTSync(
        servo_repo='servo/servo',
        wpt_repo='servo/wpt',
        downstream_wpt_repo='servo-wpt-sync/web-platform-tests',
        servo_path='./servo',
        wpt_path='./wpt',
        github_api_token=os.environ['WPT_SYNC_GITHUB_TOKEN'],
        github_api_url='https://api.github.com/',
        github_username='servo-wpt-sync',
        github_email='josh+wptsync@joshmatthews.net',
        github_name='Servo WPT Sync',
    ).run(context["event"])
    return 0 if success else 1

if __name__ == "__main__":
    sys.exit(main())
