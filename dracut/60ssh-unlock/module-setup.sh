#!/bin/sh

check() {
	require_binaries sshd dropbear dropbearconvert || return 1
	[ -x $moddir/ssh-unlock ] || return 1
	return 0
}

depends() {
	echo network
	return 0
}

install() {
	# Include sshd and configuration
	dracut_install /usr/sbin/dropbear
	/usr/bin/dropbearconvert /etc/ssh/ssh_host_ed25519_key \
		"$initdir/etc/dropbear/dropbear_ed25519_host_key"
	authorized_keys="/root/.ssh/authorized_keys"
	if [ ! -r "$authorized_keys" ]; then
		dfatal "Cannot read $authorized_keys or it does not exist."
		return 1
	fi
	mkdir -p -m 0700 "$initdir/root"
	mkdir -p -m 0700 "$initdir/root/.ssh"
	/usr/bin/install -m 0600 "$authorized_keys" "$initdir/$authorized_keys"

	inst_hook pre-udev 99 "$moddir/dropbear-start.sh"
	inst_hook pre-pivot 05 "$moddir/dropbear-stop.sh"

	# pkill is required for sshd to be killed before pivoting
	dracut_install /usr/bin/pkill

	inst "$moddir/ssh-unlock" /usr/sbin/ssh-unlock
}
