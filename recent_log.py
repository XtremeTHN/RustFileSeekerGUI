#!/usr/bin/python3

import os, sys, glob

if sys.platform == "linux":
    path = os.path.join(os.path.expanduser('~'), ".local", "share", "findermoderngui", "logs", "*.log")
elif sys.platform == "win32":
    path = os.path.join(os.path.expanduser('~'), "AppData", "Roaming", "findermoderngui", "logs", "*.log")

files = glob.glob(path)
times = {}

for x in files:
    time = os.stat(x).st_mtime
    times[x] = time

print(max(times))