#!/usr/bin/sh

umask
rm -f /tmp/umasked
touch /tmp/umasked
ls -l /tmp/umasked
chmod 777 /tmp/umasked
ls -l /tmp/umasked