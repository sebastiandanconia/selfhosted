#!/usr/bin/python3

import os
import netifaces
import iptc

DDCLIENT_CACHE="/var/cache/ddclient/ddclient.cache"
RUN_FILE="/run/minio_console_DNAT_hack"

IFACE="wan"
PORT="9000"

"""
This script does something roughly equivalent to:
iptables -t nat -N minio_dnat
iptables -t nat -I PREROUTING -j minio_dnat
iptables -t nat -F minio_dnat
iptables -t nat -A minio_dnat -d <public-ip> -p tcp --dport 9000 -j DNAT --to-destination <locally-assigned-ip>

This is sometimes necessary for the Minio Console backend to be correctly redirected
to the Minio API service when behind NAT. It seems to be relevant only with odd
networking setups, for example two NICs.
"""


def get_net_status():
    cache = open(DDCLIENT_CACHE, 'r')
    lines = cache.readlines()
    cache.close()
    #Skip initial commented lines
    while lines[0][0] == '#':
        lines = lines[1:]

    # This code supposes the first line is the one containing the IP address; if it isn't,
    # we may need to run through `lines' (top to bottom, bottom to top?) until we find an IP address.
    net_status = dict(item.split("=") for item in lines[0].split(","))

    RETURN_STATUS_KEYS = ['host', 'ip']
    return {x:net_status[x] for x in RETURN_STATUS_KEYS}


def is_first_run():
    result = not os.path.exists(RUN_FILE)

    if result:
        # Create the file if it doesn't already exist
        runfile = open(RUN_FILE, 'w')
        runfile.close()

    return result

def create_dnat_chain():
    table = iptc.Table(iptc.Table.NAT)
    dnat_chain = table.create_chain("minio_dnat")
    prert_chain = iptc.Chain(iptc.Table(iptc.Table.NAT), "PREROUTING")
    prert_rule = iptc.Rule()
    prert_target = iptc.Target(prert_rule, "minio_dnat")
    prert_rule.target = prert_target
    prert_chain.insert_rule(prert_rule)

def update_dnat(public_ip, local_ip, port):
    dnat_chain = iptc.Chain(iptc.Table(iptc.Table.NAT), "minio_dnat")
    dnat_chain.flush()
    dnat_rule = iptc.Rule()
    dnat_rule.protocol = "tcp"
    dnat_rule.dst = public_ip
    dnat_match = iptc.Match(dnat_rule, "tcp")
    dnat_match.dport = port
    dnat_rule.add_match(dnat_match)
    dnat_target = iptc.Target(dnat_rule, "DNAT")
    dnat_target.to_destination = local_ip
    dnat_rule.target = dnat_target
    dnat_chain.append_rule(dnat_rule)

def get_local_ip(iface):
    """Get (first) IPv4 address of interface"""
    addrs = netifaces.ifaddresses(iface)
    return addrs[netifaces.AF_INET][0]['addr']

if __name__ == "__main__":
    # If script hasn't run since boot, create minio_dnat chain; otherwise just update minio_dnat chain
    if is_first_run():
        create_dnat_chain()

    # TODO: Support  "--interface -i" option.
    # TODO: If IP address is provided, use that, otherwise if interface name is provided, guess the IP
    public_ip = get_net_status()['ip']
    local_ip = get_local_ip(IFACE)
    update_dnat(public_ip, local_ip, PORT)

