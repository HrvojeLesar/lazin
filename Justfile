dockerfile := "./images/build/Dockerfile.build_base"
lazin_builder_tag := "lazin:builder"
cargo_cache_volume := "lazin_cargo_cache"
container_manager := `
    if command -v docker >/dev/null 2>&1 && docker info >/dev/null 2>&1; then
        echo docker
    elif command -v podman >/dev/null 2>&1; then
        echo podman
    elif command -v docker >/dev/null 2>&1; then
        echo docker
    else
        echo podman
    fi
`

default: 
    @just --list

build_builder_image:
    {{container_manager}} build --file {{dockerfile}} --tag {{lazin_builder_tag}} .

create_cargo_cache_volume:
    {{container_manager}} volume inspect {{cargo_cache_volume}} >/dev/null 2>&1 || {{container_manager}} volume create {{cargo_cache_volume}}

cargo cmd mode: build_builder_image create_cargo_cache_volume
    mkdir -p target
    @# TODO: Remove machine id, maybe create for each container \
    {{container_manager}} run \
        --rm \
        $(test -t 2 && echo --tty) \
        --volume "$(pwd)/target":/lazin/target \
        --volume {{cargo_cache_volume}}:/usr/local/cargo/registry \
        --volume /etc/machine-id:/etc/machine-id:ro \
        {{lazin_builder_tag}} \
        cargo {{cmd}} {{ if mode == "release" { "--release" } else { "" } }}

build mode="debug":
    @just cargo build {{mode}}

clean:
    rm -rf target

clean_cache:
    {{container_manager}} volume rm {{cargo_cache_volume}}

test mode="debug":
    @just cargo test {{mode}}
