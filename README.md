# selfhosted

A collection of scripts, Dockerfiles, and configuration used for managing various self-hosted (e.g. home lab) services.

## Overview

This repository serves as a central hub for the infrastructure components used to run personal services, focusing on automation, security, and containerization.

## Components

### 🐳 Docker Containers

The `docker/` directory contains specialized images and configurations for the following services:

- **[Icecast](docker/icecast)**: A Dockerized Icecast2 server for audio streaming.
- **[Certbot](docker/certbot)**: A continuous Certbot image that uses `cron` to ensure certificates are renewed daily, ideal for NAS or headless environments.
- **[IPP Proxy](docker/ipp-proxy)**: A secure reverse proxy for the Internet Printing Protocol (IPPS) using Squid, allowing encrypted access to printers from untrusted networks.
- **MPD**: A Docker image for the Music Player Daemon.
- **[DMZ networking](docker/networking)**: ipvlan-based DMZ attachment for public-facing containers on TrueNAS SCALE.

### 🦀 Rust Projects

Custom tools written in Rust for infrastructure management:

- **[Certbot Agent](certbot-agent)**: A specialized agent for certificate management.
- **URL Mapper**: A utility used by the IPP Proxy to map secure URLs to printer endpoints.

### 📜 Scripts

Utility scripts for server maintenance and configuration:

- **MinIO Utilities**: Python scripts (`minio_console_hack.py`, `minio_scrub.py`) for managing MinIO object storage.

### 📚 Examples

- **WordPress**: A sample `docker-compose.yaml` for deploying a WordPress site.

