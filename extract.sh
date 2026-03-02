rm -rf ./docker-dist/
mkdir docker-dist
docker create --name extract mediamagician-builder --entrypoint /bin/true
docker cp extract:/MediaMagician ./docker-dist/MediaMagician
docker cp extract:/bundle ./docker-dist/bundle
docker rm extract
