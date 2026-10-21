"""Drive servoshell over WebDriver to run the rooting microbenchmarks and Speedometer.

Usage:
  harness.py micro BINARY OUT.json [--stats]
  harness.py speedometer BINARY OUT.json --suites A,B --iterations N

Every run records the peak and average RSS of the servoshell process, sampled with `ps`.
With --stats, the binary must be built with SERVO_ROOT_STATS=1; per-test rooting statistics
are parsed from its stderr.

Speedometer is served from $SPEEDOMETER_DIR (a checkout of
https://github.com/WebKit/Speedometer).
"""

import argparse
import json
import os
import re
import socket
import subprocess
import sys
import threading
import time
import urllib.request

HERE = os.path.dirname(os.path.abspath(__file__))
SPEEDOMETER_DIR = os.environ.get("SPEEDOMETER_DIR", "")


def free_port():
    with socket.socket() as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


def request(method, url, body=None, timeout=3600):
    data = json.dumps(body).encode() if body is not None else None
    req = urllib.request.Request(url, data=data, method=method)
    req.add_header("Content-Type", "application/json")
    with urllib.request.urlopen(req, timeout=timeout) as response:
        return json.loads(response.read())["value"]


class RssSampler(threading.Thread):
    def __init__(self, pid):
        super().__init__(daemon=True)
        self.pid = pid
        self.samples = []
        self.running = True

    def run(self):
        while self.running:
            try:
                out = subprocess.run(["ps", "-o", "rss=", "-p", str(self.pid)],
                                     capture_output=True, text=True).stdout.strip()
                if out:
                    self.samples.append(int(out) * 1024)
            except Exception:
                pass
            time.sleep(0.25)

    def summary(self):
        if not self.samples:
            return {}
        return {"peak_rss_mb": max(self.samples) / 2**20,
                "avg_rss_mb": sum(self.samples) / len(self.samples) / 2**20}


class Servo:
    def __init__(self, binary, stderr_path):
        self.port = free_port()
        self.stderr = open(stderr_path, "w")
        self.process = subprocess.Popen(
            [binary, "--headless", f"--webdriver={self.port}", "about:blank"],
            stdout=subprocess.DEVNULL, stderr=self.stderr)
        self.base = f"http://127.0.0.1:{self.port}"
        for _ in range(200):
            try:
                self.session = request("POST", f"{self.base}/session",
                                       {"capabilities": {}})["sessionId"]
                break
            except Exception:
                time.sleep(0.1)
        else:
            raise RuntimeError("WebDriver did not start")
        self.url = f"{self.base}/session/{self.session}"
        request("POST", f"{self.url}/timeouts", {"script": 3_600_000, "pageLoad": 600_000})
        self.rss = RssSampler(self.process.pid)
        self.rss.start()

    def navigate(self, url):
        request("POST", f"{self.url}/url", {"url": url})

    def execute(self, script, args=()):
        return request("POST", f"{self.url}/execute/sync", {"script": script, "args": list(args)})

    def close(self):
        self.rss.running = False
        try:
            request("DELETE", self.url, timeout=10)
        except Exception:
            pass
        self.process.terminate()
        try:
            self.process.wait(timeout=10)
        except subprocess.TimeoutExpired:
            self.process.kill()
        self.stderr.close()


def http_server(directory):
    port = free_port()
    process = subprocess.Popen(
        [sys.executable, "-I", "-m", "http.server", str(port), "--bind", "127.0.0.1",
         "--directory", directory],
        stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    time.sleep(0.5)
    return process, port


STATS_LINE = re.compile(r"ROOTSTATS (\S+) (\{.*\})")


def parse_stats(stderr_path):
    stats = {}
    with open(stderr_path) as f:
        for line in f:
            match = STATS_LINE.search(line)
            if match:
                stats[match.group(1)] = json.loads(match.group(2))
    return stats


def micro(args):
    server, port = http_server(HERE)
    stderr_path = args.out + ".stderr"
    servo = Servo(args.binary, stderr_path)
    try:
        servo.navigate(f"http://127.0.0.1:{port}/rooting.html")
        tests = None
        for _ in range(100):
            tests = servo.execute("return window.__tests || null;")
            if tests:
                break
            time.sleep(0.1)
            servo.navigate(f"http://127.0.0.1:{port}/rooting.html")
        if args.only:
            tests = [test for test in tests if test in args.only.split(",")]
        results = {}
        for test in tests:
            servo.execute("performance.mark('__rootstats:reset');")
            results.update(servo.execute("return window.__runAll(arguments[0]);", [[test]]))
            servo.execute("performance.mark(arguments[0]);", [f"__rootstats:{test}"])
    finally:
        servo.close()
        server.terminate()
    output = {"results": results, "memory": servo.rss.summary()}
    if args.stats:
        output["root_stats"] = parse_stats(stderr_path)
    with open(args.out, "w") as f:
        json.dump(output, f, indent=2)


def speedometer(args):
    if not SPEEDOMETER_DIR:
        sys.exit("Set SPEEDOMETER_DIR to a Speedometer checkout.")
    server, port = http_server(SPEEDOMETER_DIR)
    stderr_path = args.out + ".stderr"
    servo = Servo(args.binary, stderr_path)
    try:
        query = f"?startAutomatically&iterationCount={args.iterations}"
        if args.suites:
            query += f"&suites={args.suites}"
        servo.navigate(f"http://127.0.0.1:{port}/{query}")
        servo.execute("performance.mark('__rootstats:reset');")
        while not servo.execute("return !!(window.benchmarkClient && benchmarkClient._hasResults);"):
            time.sleep(1)
        servo.execute("performance.mark('__rootstats:speedometer');")
        results = servo.execute("return benchmarkClient._measuredValuesList;")
    finally:
        servo.close()
        server.terminate()
    output = {"results": results, "memory": servo.rss.summary()}
    if args.stats:
        output["root_stats"] = parse_stats(stderr_path)
    with open(args.out, "w") as f:
        json.dump(output, f, indent=2)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("mode", choices=["micro", "speedometer"])
    parser.add_argument("binary")
    parser.add_argument("out")
    parser.add_argument("--stats", action="store_true")
    parser.add_argument("--suites", default="")
    parser.add_argument("--iterations", type=int, default=5)
    parser.add_argument("--only", default="")
    args = parser.parse_args()
    {"micro": micro, "speedometer": speedometer}[args.mode](args)


if __name__ == "__main__":
    main()
