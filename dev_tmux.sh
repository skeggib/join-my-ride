#!/bin/sh

tmux new-session \; \
    send-keys 'cd frontend/; cargo make watch' C-m \; \
    split-window -h \; \
    send-keys 'cd backend/; cargo watch -x run' C-m