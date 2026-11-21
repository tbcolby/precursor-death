"""Renode capture sequence for Game of Death.

Run from xous-dev-toolkit:
    python3 scripts/renode_capture.py --app death --script ../precursor-death/capture.py

Contract: run(ctl, outdir) -> number of failed steps. The app is already launched.
"""
import os


def run(ctl, outdir) -> int:
    failures = 0

    def shot(name):
        nonlocal failures
        if not ctl.try_screenshot(os.path.join(outdir, name)):
            failures += 1

    shot("01_rules.png")
    ctl.inject_line("")  # Enter: start Life from its soup
    ctl.sleep(2.0)
    shot("02_running.png")
    ctl.timed_key("BackSpace", after=1.0)  # back to the rule list
    for _ in range(12):
        ctl.timed_key("Down", after=0.5)
    ctl.inject_line("")  # Enter on Custom: the B/S editor
    ctl.sleep(1.0)
    shot("03_rule_editor.png")
    return failures
