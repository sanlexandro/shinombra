CC := gcc
REQUIRED_PKGS := libpipewire-0.3 libspa-0.2 libportal gobject-2.0 glib-2.0 gio-2.0

# Gather flags via pkg-config (will be evaluated at make runtime)
PKG_CFLAGS := $(shell pkg-config --cflags $(REQUIRED_PKGS) 2>/dev/null || true)
PKG_LIBS := $(shell pkg-config --libs $(REQUIRED_PKGS) 2>/dev/null || true)

CFLAGS := -Wall -Wextra -O2 -fPIC $(PKG_CFLAGS)
LDFLAGS := $(PKG_LIBS)

SRC := pw-capture-video-test.c
OBJ := $(SRC:.c=.o)
BUILD_PATH := ../build
BIN := $(BUILD_PATH)/pw-capture-video-test

# Ensure required system pkg-config packages are available before building
all: check-pkg $(BIN)

$(BUILD_PATH):
	@mkdir -p $(BUILD_PATH)

$(BIN): $(BUILD_PATH) $(OBJ)
	@$(CC) $(OBJ) -o $@ $(LDFLAGS)

%.o: %.c
	@$(CC) $(CFLAGS) -c $< -o $@

run: $(BIN)
	@./$(BIN)


clean:
	@rm -f $(OBJ) $(BIN)

.PHONY: all clean run check-pkg

check-pkg:
	@if [ -z "$(PKG_CFLAGS)" ] || [ -z "$(PKG_LIBS)" ]; then exit 1; fi
