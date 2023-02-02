#!/usr/bin/python3

# Script that should be run from the root of the repository.
# It validates that the GRPC server with a set of binaries from the UPF platform

import os
import subprocess
import argparse
from pathlib import Path


parser = argparse.ArgumentParser(description="Run GRPC server.")
parser.add_argument(
    "--executable", help="Path to the executable to run", default=None, nargs="?"
)

args = parser.parse_args()
executable = args.executable if args.executable else "target/ci/up-server"

if not args.executable:
    build_result = os.system("cargo build --profile ci --bin up-server")
    if build_result != 0:
        exit(1)

    solver = "target/ci/up-server"
else:
    solver = os.path.abspath(args.executable)

solver_cmd = solver + " --address 0.0.0.0:2222 --file-path {instance}"

problem_dir = Path("./planning/ext/up/bins/problems/").resolve()
problem_files = list(map(str, list(problem_dir.iterdir())))

failed = 0
for problem_file in problem_files:
    cmd = solver_cmd.format(instance=problem_file).split(" ")
    print("\nSolving instance: " + problem_file)
    print("Command: " + " ".join(cmd))
    solver_run = subprocess.run(cmd, stdout=subprocess.PIPE, universal_newlines=True)
    if solver_run.returncode != 0:
        failed += 1
if failed != 0:
    print(f"\n===== {failed} errors on {len(problem_files)} problems =====")
exit(failed)
