pingd -- simple ping-logging daemon with reporting page
=======================================================

`pingd` is a simple daemon that pings a specified host once per 10 seconds,
records ping status (reply or no reply, and latency if replied) in a SQLite
database.  It also includes a very simple dynamic status/reporting page served
over HTTP.

`pingd` may be useful to monitor whether a host is up and keep a record of its
network reachability. It may also be useful to record the status history of a
home Internet connection to, for example, provide evidence of poor connectivity
on an ISP's network.

Compiling
---------

`pingd` may be compiled via Cargo, as per normal for Rust programs:

    $ cargo build --release
    $ target/release/pingd

Usage
-----

Launch `pingd` with the following three command-line arguments:

    $ pingd [sqlite-database-file] [hostname-to-ping] [serve-address]

For example:

    $ pingd /var/pingd/ping.db myhost localhost:8080

This will ping `myhost` once per 10 seconds, save the results indefinitely in
`/var/pingd/ping.db` (you may want to rotate this database occasionally), and
serve the results at `http://localhost:8080/`.

Installing
----------

`make install` will install the daemon, an Upstart boot-time startup script
(for Ubuntu machines), and a configuration file at `/etc/pingd.conf`. Edit this
file to set the hostname that will be pinged by `pingd` and to set a port on
which to listen for the HTTP interface. By default, `pingd` listens on port
8080 on `localhost` only. You may find it useful to install `pingd` on a server
and then proxy this (e.g., via nginx) to a sub-path on your main web s erver.

HTTP interface
--------------

`pingd` provides a read-only interface over HTTP displaying the latest ping
results.

The last 600 pings (by default) are shown on the main page at the serving
address/port. If you would like to see a different number of pings, or all
pings ever recorded, view `/<n>` or `/all`, respectively (e.g.,
`http://localhost:8080/all`).

All reporting pages show a simple table that displays, for each ping, the
hostname, timestamp, and latency of response (or "NO RESPONSE").

License
-------

`pingd` is Copyright (c) 2016 Chris Fallin &lt;cfallin@c1f.net&gt; and is
released under the MIT License.
