#!/bin/sh

check() {
	# Dropbear could be an option but I need ed25519 support.
	require_binaries sshd || return 1
	[ -x $moddir/crypt-unlock ] || return 1
	return 0
}

depends() {
	echo network
	return 0
}

install() {
	# Include sshd and configuration
	dracut_install /usr/sbin/sshd /etc/ssh/ssh_host_ed25519_key.pub
	/usr/bin/install -m 0600 /etc/ssh/ssh_host_ed25519_key \
		"$initdir/etc/ssh/ssh_host_ed25519_key"
	inst "$moddir/sshd_config" /etc/ssh/sshd_config
	authorized_keys="/root/.ssh/authorized_keys"
	if [ ! -r "$authorized_keys" ]; then
		dfatal "Cannot read $authorized_keys or it does not exist."
		return 1
	fi
	mkdir -p -m 0700 "$initdir/root"
	mkdir -p -m 0700 "$initdir/root/.ssh"
	/usr/bin/install -m 0600 "$authorized_keys" "$initdir/$authorized_keys"

	# Required for sshd's privilege separation
	grep '^sshd:' /etc/passwd >> "$initdir/etc/passwd"
	grep '^sshd:' /etc/group >> "$initdir/etc/group"
	mkdir -p -m 0755 "$initdir/var/empty/sshd"

	inst_hook pre-udev 99 "$moddir/sshd-start.sh"
	inst_hook pre-pivot 05 "$moddir/sshd-stop.sh"

	# pkill is required for sshd to be killed before pivoting
	dracut_install /usr/bin/pkill

	inst "$moddir/crypt-unlock" /usr/sbin/crypt-unlock
}
