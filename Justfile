build-release:
    trunk build --release

publish: build-release
    rsync -ahP dist/* kcloud:~/kcloud/pairandomizer/
