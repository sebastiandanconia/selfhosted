# Certbot Docker Image

The official Certbot Docker images are generally intended to be invoked as needed for certificate renewal. By contrast, this image is intended to stay running continuously, calling `certbot` and related scripts at least daily via `cron` jobs. This is useful for running within a NAS or similar environment.