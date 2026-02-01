# Minimal HTTP_LISTEN test
SETS 255, "9092"
TEXEC 0x8000, 255, 0        # HTTP_INIT

SETS 255, "Started on 9092"
TEXEC 0x5000, 255, 0

Loop:
    SETS 255, "Waiting..."
    TEXEC 0x5000, 255, 0

    TEXEC 0x8008, 255, 0    # HTTP_LISTEN -> R0

    SETS 255, "Got request!"
    TEXEC 0x5000, 255, 0
    TEXEC 0x5000, 0, 0      # Print request

    SETS 255, "Hello from L-0!"
    TEXEC 0x8009, 255, 0    # HTTP_SEND

    JMP Loop
