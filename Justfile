WIDTH := "1920"
HEIGHT := "1080"
FRAMES := "1200"
FPS := "30"
MAX_ITER := "2000"
ZOOM_START := "1.0"
#ZOOM_END := "1e-6"
ZOOM_END := "1e-20"
OUT_DIR := "out/frames"
OUT_VIDEO := "out/mandelbrot.mp4"

default:
  @just --list

render:
  cargo run --release -- \
    --width {{WIDTH}} \
    --height {{HEIGHT}} \
    --frames {{FRAMES}} \
    --fps {{FPS}} \
    --max-iter {{MAX_ITER}} \
    --zoom-start {{ZOOM_START}} \
    --zoom-end {{ZOOM_END}} \
    --out-dir {{OUT_DIR}}

video:
  ffmpeg -y -framerate {{FPS}} \
    -i {{OUT_DIR}}/frame_%06d.png \
    -c:v libx264 \
    -pix_fmt yuv420p \
    {{OUT_VIDEO}}

all: render video

clean-frames:
  rm -f {{OUT_DIR}}/frame_*.png
