#include "audio_test.h"
#include "vand_audio.h"

#ifdef VAND_AUDIO_GENERATED
#include "generated_audio.h"
#endif

#ifndef VAND_AUDIO_GENERATED
static const VandAudioEvent audioTestPattern[] =
{
    {12, 0x43B, 1, 15, 0x35B, 4, 10, 0x000, 0, 0, 0x000, 0, 0, 0x000, 0, 0, 0, VAND_AUDIO_CLASS_BASS, 255, 0},
    {6, 0x43B, 1, 12, 0x3C5, 4, 12, 0x000, 0, 0, 0x000, 0, 0, 0x000, 0, 0, 4, VAND_AUDIO_CLASS_TONAL, 255, 0},
    {6, 0x4B8, 1, 13, 0x43B, 4, 12, 0x000, 0, 0, 0x000, 0, 0, 0x000, 0, 0, 10, VAND_AUDIO_CLASS_DRUM, 255, 0},
    {12, 0x3C5, 2, 14, 0x4B8, 4, 10, 0x000, 0, 0, 0x000, 0, 0, 0x000, 0, 0, 0, VAND_AUDIO_CLASS_BASS, 255, 0},
    {6, 0x35B, 2, 12, 0x55F, 4, 13, 0x000, 0, 0, 0x000, 0, 0, 0x000, 0, 0, 3, VAND_AUDIO_CLASS_TONAL, 255, 0},
    {6, 0x35B, 2, 10, 0x5FE, 4, 11, 0x000, 0, 0, 0x000, 0, 0, 0x000, 0, 0, 12, VAND_AUDIO_CLASS_SAMPLE, 255, 0},
    {12, 0x43B, 2, 15, 0x55F, 4, 10, 0x000, 0, 0, 0x000, 0, 0, 0x000, 0, 0, 0, VAND_AUDIO_CLASS_BASS, 255, 0},
    {6, 0x43B, 2, 12, 0x4B8, 4, 12, 0x000, 0, 0, 0x000, 0, 0, 0x000, 0, 0, 2, VAND_AUDIO_CLASS_TONAL, 255, 0},
    {6, 0x4B8, 2, 13, 0x43B, 4, 12, 0x000, 0, 0, 0x000, 0, 0, 0x000, 0, 0, 15, VAND_AUDIO_CLASS_DRUM, 255, 0},
    {18, 0x000, 0, 0, 0x000, 0, 0, 0x000, 0, 0, 0x000, 0, 0, 0x000, 0, 0, 0, VAND_AUDIO_CLASS_SILENCE, 255, 0},
};
#define AUDIO_TEST_EVENTS audioTestPattern
#define AUDIO_TEST_EVENT_COUNT (sizeof(audioTestPattern) / sizeof(audioTestPattern[0]))
#define AUDIO_TEST_SOURCE_TEXT "BUILTIN PATTERN"
#else
#define AUDIO_TEST_EVENTS generatedAudioEvents
#define AUDIO_TEST_EVENT_COUNT generatedAudioEventCount
#define AUDIO_TEST_SOURCE_TEXT "GENERATED ARRANGEMENT"
#endif

static void drawAudioTest(void)
{
    VDP_clearPlane(BG_A, TRUE);
    VDP_clearPlane(BG_B, TRUE);
    VDP_setTextPalette(PAL0);
    VDP_drawText("VANDALER AUDIO LAB", 10, 4);
    VDP_drawText("YM2612 FM + PSG NOISE", 8, 7);
    VDP_drawText("AUTO PLAY ON BOOT", 11, 10);
    VDP_drawText("START: PLAY / STOP", 9, 12);
    VDP_drawText("A: RESTART", 14, 13);
    VDP_drawText("B: STOP", 16, 15);
    VDP_drawText(AUDIO_TEST_SOURCE_TEXT, 9, 17);
    VDP_drawText("BUILD: make audio-test", 8, 21);
}

static void drawStatus(bool playing, u16 index)
{
    char indexText[8];

    uintToStr(index, indexText, 3);
    VDP_clearText(12, 18, 18);
    VDP_drawText(playing ? "PLAYING" : "STOPPED", 12, 18);
    VDP_drawText("EVENT", 12, 19);
    VDP_drawText(indexText, 18, 19);
}

int AudioTest_main(void)
{
    VandAudioPlayer player;
    u16 previousJoy = 0;
    bool playing = FALSE;

    VDP_setScreenWidth320();
    VDP_setPlaneSize(64, 32, TRUE);
    SPR_init();
    PSG_reset();
    VandAudio_init();
#ifdef VAND_AUDIO_GENERATED
    VandAudio_setDacBank(generatedAudioDacSamples, generatedAudioDacLengths, generatedAudioDacRates, generatedAudioDacCount);
#else
    VandAudio_setDacBank(NULL, NULL, NULL, 0);
#endif
    drawAudioTest();

    VandAudio_start(&player, AUDIO_TEST_EVENTS, AUDIO_TEST_EVENT_COUNT, TRUE);
    playing = TRUE;

    while (TRUE)
    {
        const u16 joy = JOY_readJoypad(JOY_1);
        const u16 pressed = joy & ~previousJoy;

        if (pressed & BUTTON_START)
        {
            if (playing)
            {
                VandAudio_stop(&player);
                playing = FALSE;
            }
            else
            {
                VandAudio_start(&player, AUDIO_TEST_EVENTS, AUDIO_TEST_EVENT_COUNT, TRUE);
                playing = TRUE;
            }
        }
        if (pressed & BUTTON_A)
        {
            VandAudio_start(&player, AUDIO_TEST_EVENTS, AUDIO_TEST_EVENT_COUNT, TRUE);
            playing = TRUE;
        }
        if (pressed & BUTTON_B)
        {
            VandAudio_stop(&player);
            playing = FALSE;
        }

        VandAudio_update(&player);
        drawStatus(player.playing, player.index);

        previousJoy = joy;
        SPR_update();
        SYS_doVBlankProcess();
    }

    return 0;
}
