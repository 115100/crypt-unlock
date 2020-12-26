#!/bin/sh
/usr/sbin/dropbear -wsgjk \
	-p 2222 \
	-c /usr/sbin/ssh-unlock
