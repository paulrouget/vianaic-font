all:
	cargo run --release

download:
	curl --header "Cookie: PHPSESSID=52d24cd2ce0e22458462977c35e8b93b" http://2ttf.com/download/CFuDGXr3uSa/otf -o vianaic.src.otf

clean:
	rm -rf target
