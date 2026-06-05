# Icecast Docker Container

Icecast2 Dockerfile

## Run

Run with default password, export port 8000

```bash
docker run -p 8000:8000 sebastiandanconia/icecast
$BROWSER localhost:8000
```

Run with custom password

```bash
docker run -p 8000:8000 -e ICECAST_SOURCE_PASSWORD=aaaa -e ICECAST_ADMIN_PASSWORD=bbbb -e ICECAST_PASSWORD=cccc -e ICECAST_RELAY_PASSWORD=dddd -e ICECAST_HOSTNAME=noise.example.com sebastiandanconia/icecast
```

Run with custom configuration

```bash
docker run -p 8000:8000 -v /local/path/to/icecast/config:/etc/icecast2 sebastiandanconia/icecast
docker run -p 8000:8000 -v /local/path/to/icecast.xml:/etc/icecast2/icecast.xml sebastiandanconia/icecast
```

Extends Dockerfile

```Dockerfile
FROM sebastiandanconia/icecast
ADD ./icecast.xml /etc/icecast2
```

Docker-compose

```yaml
icecast:
  image: sebastiandanconia/icecast
  volumes:
  - logs:/var/log/icecast2
  - /etc/localtime:/etc/localtime:ro
  environment:
  - ICECAST_SOURCE_PASSWORD=aaa
  - ICECAST_ADMIN_PASSWORD=bbb
  - ICECAST_PASSWORD=ccc
  - ICECAST_RELAY_PASSWORD=ddd
  - ICECAST_HOSTNAME=noise.example.com
  ports:
  - 8000:8000
```
