# Icecast Docker Image

Multi-architecture Docker image for [Icecast](https://icecast.org/) streaming server.

Derived from [moul/docker-icecast](https://github.com/moul/docker-icecast).

## Quick Start

Run MPD with default settings:
```bash
docker pull sebastiandanconia/icecast
docker run -p 8000:8000 sebastiandanconia/icecast
```

OR

Run MPD with some custom settings:
```bash
docker run -p 8000:8000 -e ICECAST_SOURCE_PASSWORD=aaaa -e ICECAST_ADMIN_PASSWORD=bbbb -e ICECAST_PASSWORD=cccc -e ICECAST_RELAY_PASSWORD=dddd -e ICECAST_HOSTNAME=noise.example.com sebastiandanconia/icecast
```

OR

Run MPD with fully customizable settings:
```bash
docker run -p 8000:8000 -v /local/path/to/icecast.xml:/etc/icecast2/icecast.xml sebastiandanconia/icecast
```

OR

Run MPD using `docker compose`:
```yaml
icecast:
  image: sebastiandanconia/icecast
  volumes:
    # Optional, but useful for advanced configurations
    - /srv/docker/volumes/icecast/icecast.xml:/etc/icecast2/icecast.xml
    - logs:/var/log/icecast2
  environment:
    # Uses the host's $TZ variable, or defaults to UTC
    - TZ=${TZ:-UTC}
    - ICECAST_SOURCE_PASSWORD=aaa
    - ICECAST_ADMIN_PASSWORD=bbb
    - ICECAST_PASSWORD=ccc
    - ICECAST_RELAY_PASSWORD=ddd
    - ICECAST_HOSTNAME=noise.example.com
  ports:
    - 8000:8000
```

OR

Build your configuration file into an image:
```Dockerfile
FROM sebastiandanconia/icecast
ADD ./icecast.xml /etc/icecast2
```

## Supported Architectures

| Architecture | Tag |
|--------------|-----|
| x86_64 (amd64) | `sebastiandanconia/icecast:latest` |
| ARM64 (aarch64) | `sebastiandanconia/icecast:latest` |

Docker automatically selects the correct architecture for your system.

## Building Locally

### Multi-Architecture Build (requires Docker Buildx)

```bash
# Create a builder instance (one-time setup)
docker buildx create --name multiarch --driver docker-container --use

# Build and push multi-arch image
docker buildx build \
  --platform linux/amd64,linux/arm64 \
  --tag sebastiandanconia/icecast:latest \
  --push \
  .
```

### Single Architecture Build

```bash
docker build -t sebastiandanconia/icecast:latest .
docker push sebastiandanconia/icecast:latest
```

## CI/CD

This repository uses GitHub Actions to automatically build and push multi-architecture images.

### Required Secrets

Configure these in your GitHub repository settings → Secrets and variables → Actions:

| Secret | Description |
|--------|-------------|
| `DOCKERHUB_USERNAME` | Your Docker Hub username |
| `DOCKERHUB_TOKEN` | Docker Hub access token ([create one here](https://hub.docker.com/settings/security)) |

## Configuration

Mount your Icecast configuration:

```bash
docker run -p 8000:8000 \
  -v /path/to/icecast.xml:/etc/icecast2/icecast.xml \
  sebastiandanconia/icecast
```

## License

See original project: https://github.com/moul/docker-icecast
