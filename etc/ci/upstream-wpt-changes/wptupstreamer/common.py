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

UPSTREAMABLE_PATH = "tests/wpt/web-platform-tests/"
NO_SYNC_SIGNAL = "[no-wpt-sync]"

OPENED_NEW_UPSTREAM_PR = (
    "🤖 Opened new upstream WPT pull request ({upstream_pr}) "
    "with upstreamable changes."
)
UPDATED_EXISTING_UPSTREAM_PR = (
    "📝 Transplanted new upstreamable changes to existing "
    "upstream WPT pull request ({upstream_pr})."
)
UPDATED_TITLE_IN_EXISTING_UPSTREAM_PR = (
    "✍ Updated existing upstream WPT pull request ({upstream_pr}) title and body."
)
CLOSING_EXISTING_UPSTREAM_PR = (
    "🤖 This change no longer contains upstreamable changes to WPT; closed existing "
    "upstream pull request ({upstream_pr})."
)
NO_UPSTREAMBLE_CHANGES_COMMENT = (
    "👋 Downstream pull request ({servo_pr}) no longer contains any upstreamable "
    "changes. Closing pull request without merging."
)
COULD_NOT_APPLY_CHANGES_DOWNSTREAM_COMMENT = (
    "🛠 These changes could not be applied onto the latest upstream WPT. "
    "Servo's copy of the Web Platform Tests may be out of sync."
)
COULD_NOT_APPLY_CHANGES_UPSTREAM_COMMENT = (
    "🛠 Changes from the source pull request ({servo_pr}) can no longer be "
    "cleanly applied. Waiting for a new version of these changes downstream."
)
COULD_NOT_MERGE_CHANGES_DOWNSTREAM_COMMENT = (
    "⛔ Failed to properly merge the upstream pull request ({upstream_pr}). "
    "Please address any CI issues and try to merge manually."
)
COULD_NOT_MERGE_CHANGES_UPSTREAM_COMMENT = (
    "⛔ The downstream PR has merged ({servo_pr}), but these changes could not "
    "be merged properly. Please address any CI issues and try to merge manually."
)


def wpt_branch_name_from_servo_pr_number(servo_pr_number):
    return f"servo_export_{servo_pr_number}"
