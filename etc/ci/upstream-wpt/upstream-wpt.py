#!/usr/bin/env python

import json
import os
import logging

from sync import WPTSync

context = json.loads(os.environ['GITHUB_CONTEXT'])
logging.getLogger().level = logging.INFO

WPTSync(
    servo_repo='mrobinson/servo',
    wpt_repo='mrobinson/wpt',
    downstream_wpt_repo='mrobinson/wpt',
    servo_path='./servo',
    wpt_path='./wpt',
    github_api_token=os.environ['GITHUB_TOKEN'],
    github_api_url='https://api.github.com/',
    github_username='mrobinson',
    github_email='mrobinson@igalia.com',
    github_name='Servo WPT Sync',
).run(context["event"])
