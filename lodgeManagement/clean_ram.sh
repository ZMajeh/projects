#!/bin/bash
# Script to clean cached RAM (pagecache, dentries, and inodes)
# Note: This requires sudo privileges.

echo "Synchronizing cached writes to disk..."
sync

echo "Clearing PageCache, dentries, and inodes..."
if sudo sh -c 'echo 3 > /proc/sys/vm/drop_caches'; then
    echo "RAM cache cleaned successfully."
else
    echo "Failed to clean RAM cache. Please ensure you have sudo privileges."
fi
