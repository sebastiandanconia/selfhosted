# DMZ Docker networking on TrueNAS SCALE

How public-facing containers are attached **only** to a DMZ VLAN, so isolation is enforced by the network and firewall rather than by Docker port publishing on the host LAN.

Addresses, interface names, and VLAN IDs below are placeholders. Substitute your own DMZ values.

## Goals

- Public-facing workloads get real addresses on a dedicated DMZ subnet (IPv4 and optionally IPv6).
- Those containers are **not** on the default Docker bridge or the trusted LAN.
- East-west and north-south policy lives on the router/firewall between VLANs (allow only the ports you intend; deny DMZ → LAN by default).
- Compose apps on TrueNAS SCALE can join that network even though the Apps UI does not expose a first-class “put this app on VLAN X” control.

## Why this shape

TrueNAS SCALE (Electric Eel and later) runs apps as Docker Compose stacks. Compose’s default `bridge` network is host-local: published ports land on the NAS’s LAN address and every container that shares a user-defined bridge can usually reach the others. That is the wrong trust boundary for internet-facing services.

Docker’s **ipvlan** (L2) driver binds a Docker network to an existing parent interface—in this setup, the NAS’s DMZ VLAN interface. Containers on that network appear as ordinary hosts on the DMZ subnet. The firewall in front of that VLAN is then the primary control plane.

ipvlan L2 is preferred here over macvlan:

- One MAC on the parent (fewer switch/AP CAM and port-security surprises).
- Works cleanly on VLAN subinterfaces that TrueNAS already manages.
- macvlan remains a drop-in alternative if you need unique MACs per container.

This is close to optimal for “containers *live in* the DMZ” on a single TrueNAS host. The main alternative architectures are different products, not better wiring:

| Approach | When it is better |
| --- | --- |
| Reverse proxy in DMZ, apps on LAN | You want only the proxy exposed and apps never on a public VLAN |
| Separate DMZ VM / host | You need a stronger blast-radius boundary than containers on the NAS |
| Host networking + bind to a DMZ host IP | Quick hack; weak isolation between containers |
| Default bridge + published ports | Simple LAN services only; not a DMZ |

There is no cleaner Docker-native way to put individual Compose services on a physical VLAN than ipvlan/macvlan. The only “hack” is how the shared network object is created under TrueNAS Apps (below).

## Prerequisites

1. A DMZ VLAN and subnet on your router/firewall.
2. The NAS has a VLAN interface on that segment (example name: `vlanDMZ`) and can reach the DMZ gateway.
3. Firewall rules that treat the DMZ as untrusted relative to the LAN (default-deny DMZ → LAN; explicit WAN → DMZ allows for published services).
4. A reserved address pool for containers that does not overlap routers, VIPs, or static infrastructure on the VLAN.

## Architecture

```
Internet
   |
[ Firewall / router ]
   |                 \
   | WAN→DMZ allows   \ default-deny DMZ→LAN
   v                   v
DMZ VLAN                 Trusted LAN VLAN
10.99.30.0/24            (NAS management, storage clients, …)
   |
   | parent: vlanDMZ
   v
Docker ipvlan network "dmz"   (attachable, external to app stacks)
   |
   +-- rustfs-dmz   10.99.30.50
   +-- other public apps …
```

Containers use **static** DMZ addresses so firewall rules and DNS stay stable. Do **not** also attach these services to `bridge` or LAN networks. Do **not** publish `ports:` to the host unless you deliberately want a second path onto the NAS LAN IP.

### Host ↔ container caveat

ipvlan/macvlan endpoints on the same parent are not reachable from the host’s own IP stack by default. That is usually desirable for isolation. Management of DMZ apps should go through the DMZ address (from a host that is allowed), a jump host, or a separate management path you design on purpose—not through `localhost` port maps on the NAS.

## Step 1: Define the shared DMZ network

Compose must create the network once. App stacks then reference it as `external: true`. TrueNAS Apps expects a Compose file with services, so a zero-replica placeholder service is used solely to materialize the network without running a workload. `scale: 0` is a Compose service field that keeps that placeholder at zero replicas.

Deploy this as its own stack (custom app). Bring it up once; leave it installed so the network object remains defined the way you expect.

```yaml
# dmz-net.yaml — creates the shared DMZ network; runs no containers
services:
  dmz-net-placeholder:
    image: alpine:latest
    scale: 0
    networks:
      dmz:
        ipv4_address: 10.99.30.62
    command: ["true"]

networks:
  dmz:
    name: dmz
    driver: ipvlan
    attachable: true
    enable_ipv6: true
    driver_opts:
      parent: vlanDMZ
      # ipvlan_mode: l2   # default; L2 bridges onto the parent VLAN
    ipam:
      config:
        - subnet: 10.99.30.0/24
          ip_range: 10.99.30.32/27
          gateway: 10.99.30.1
        - subnet: 2001:db8:30::/64
          gateway: 2001:db8:30::1
```

Notes on the definition:

- **`name: dmz`** — fixed name so every app stack can find the same network.
- **`attachable: true`** — required so other Compose projects can join.
- **`parent`** — the TrueNAS VLAN interface for the DMZ (not a bridge you invent in Docker).
- **`ip_range`** — optional but recommended; limits Docker’s dynamic allocations to a slice of the subnet. Prefer static `ipv4_address` on real services either way.
- **Do not set `external: true` here** — this file *defines* the network. Clients set `external: true`.
- IPv6 is optional; drop the second IPAM entry and `enable_ipv6` if you are IPv4-only.
- The placeholder’s static address is irrelevant while `scale: 0`; it only needs to be a valid address in the pool if you ever scale it up for debugging.

### Shell alternative

If you manage Docker from a shell instead of an Apps stack, the same network is:

```bash
docker network create \
  -d ipvlan \
  --subnet 10.99.30.0/24 \
  --ip-range 10.99.30.32/27 \
  --gateway 10.99.30.1 \
  --ipv6 \
  --subnet 2001:db8:30::/64 \
  --gateway 2001:db8:30::1 \
  -o parent=vlanDMZ \
  --attachable \
  dmz
```

That is slightly cleaner operationally; the Compose form exists because TrueNAS Apps is Compose-centric.

## Step 2: Attach a public app (RustFS example)

Client stacks declare the network as external and pin a DMZ address. No `ports:` mapping: clients reach the service at the container’s DMZ IP (or via firewall DNAT to that IP).

```yaml
# rustfs on the DMZ only — example addresses and paths
services:
  rustfs-dmz:
    container_name: rustfs-dmz
    image: rustfs/rustfs:latest
    restart: unless-stopped
    command:
      - /entrypoint.sh
      - rustfs
      - --address
      - :9000
      - --console-enable
      - /data
    environment:
      RUSTFS_ACCESS_KEY: rustfsadmin
      RUSTFS_SECRET_KEY: D3aY62VJN15iwErBgGFwMYXe7GRVjv8upqEytj6PwpdE
      RUSTFS_SERVER_DOMAINS: rustfs.example.com
      RUSTFS_CONSOLE_ENABLE: "true"
    volumes:
      - /mnt/tank/Applications/volumes/rustfs/data:/data
    networks:
      dmz:
        ipv4_address: 10.99.30.50
    # Intentionally no ports: — traffic hits 10.99.30.50 directly on the DMZ
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:9000/health"]
      interval: 30s
      timeout: 20s
      retries: 3

networks:
  dmz:
    name: dmz
    external: true
```

Point DNS for `rustfs.example.com` at `10.99.30.50` (or at a firewall IP address that DNATs to it). Open only the needed ports on the firewall (typically 9000 + 9001 for S3/API and console, as configured). Keep credentials out of the Compose file if possible (env file or your secret mechanism).

Any other public container follows the same pattern: single network attachment to `dmz`, static IP, no host port publish, no extra LAN networks.

## Isolation checklist

- [ ] DMZ containers have **only** the `dmz` network (no default bridge, no LAN bridge).
- [ ] No `ports:` / host publishes for DMZ services unless you accept exposure on the NAS LAN IP.
- [ ] Firewall: WAN → DMZ allowlist by IP and port; DMZ → LAN default deny; DMZ → internet only if required.
- [ ] Static IPs documented and excluded from DHCP on the VLAN.
- [ ] Volumes for DMZ apps are not shared with trusted-LAN containers unless you accept that data path.
- [ ] Management access to consoles is authenticated and, if possible, limited by source address.

## Operational tips

- **Create the network stack first**, then app stacks that set `external: true`.
- **Do not** `docker compose down` the network definition stack in a way that removes the `dmz` network while apps still reference it; tear down clients first or use `docker network rm` only when idle.
- **Debugging connectivity**: temporarily set the placeholder `scale` to 1 with `tty`/`sleep infinity`, attach, and use `ip addr` / `ping` from inside the DMZ namespace—or run a one-off container with `--network dmz`.
- **IPv6**: if you enable it, mirror the same firewall policy as IPv4; an open v6 path undoes v4 discipline.
- **TrueNAS upgrades**: VLAN interfaces and Docker networks sometimes need a quick check after major SCALE upgrades; re-apply the network stack if `dmz` disappeared.

## Summary

Public containers are ordinary DMZ hosts via an attachable ipvlan network on the NAS’s DMZ VLAN interface. Compose’s `external` network plus a zero-replica definition stack is the TrueNAS-friendly way to create that network once and reuse it. Isolation is primarily VLAN + firewall policy; Docker’s job is only to keep those processes off the trusted networks.
