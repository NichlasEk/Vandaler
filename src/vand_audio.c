#include "vand_audio.h"

#define VAND_AUDIO_FM_BASS_CH 0
#define VAND_AUDIO_FM_LEAD_CH 1
#define VAND_AUDIO_FM_KEY_ALL 0xF0

static bool audioReady = FALSE;

static void ymKey(u8 channel, bool on)
{
    YM2612_writeReg(0, 0x28, (on ? VAND_AUDIO_FM_KEY_ALL : 0x00) | (channel & 0x03));
}

static void ymSetFrequency(u8 channel, u16 fnum, u8 block)
{
    YM2612_writeReg(0, 0xA4 + channel, ((block & 0x07) << 3) | ((fnum >> 8) & 0x07));
    YM2612_writeReg(0, 0xA0 + channel, fnum & 0xFF);
}

static void ymSetLevel(u8 channel, u8 level)
{
    const u8 clamped = level > 15 ? 15 : level;
    const u8 carrierTl = 127 - (clamped * 7);

    YM2612_writeReg(0, 0x40 + channel, 112);
    YM2612_writeReg(0, 0x44 + channel, 112);
    YM2612_writeReg(0, 0x48 + channel, 112);
    YM2612_writeReg(0, 0x4C + channel, carrierTl);
}

static void ymInitVoice(u8 channel, bool bass)
{
    const u8 detuneMul[4] = {bass ? 0x01 : 0x02, bass ? 0x02 : 0x03, 0x01, 0x01};
    const u8 rates[4] = {0x1F, bass ? 0x18 : 0x12, bass ? 0x14 : 0x10, bass ? 0x16 : 0x14};
    const u8 release[4] = {0x0F, 0x0F, 0x0F, 0x0F};
    const u8 sustain[4] = {0x08, 0x0A, 0x0C, 0x08};
    const u8 offsets[4] = {0x00, 0x04, 0x08, 0x0C};
    u8 op;

    YM2612_writeReg(0, 0xB0 + channel, bass ? 0x32 : 0x31);
    YM2612_writeReg(0, 0xB4 + channel, 0xC0);

    for (op = 0; op < 4; op++)
    {
        const u8 reg = offsets[op] + channel;
        YM2612_writeReg(0, 0x30 + reg, detuneMul[op]);
        YM2612_writeReg(0, 0x50 + reg, rates[op]);
        YM2612_writeReg(0, 0x60 + reg, 0x05);
        YM2612_writeReg(0, 0x70 + reg, 0x02);
        YM2612_writeReg(0, 0x80 + reg, release[op]);
        YM2612_writeReg(0, 0x90 + reg, sustain[op]);
    }

    ymSetLevel(channel, 0);
}

static void psgSetNoiseLevel(u8 level, u8 kind)
{
    const u8 clamped = level > 15 ? 15 : level;
    const u8 envelope = clamped == 0 ? PSG_ENVELOPE_MIN : 15 - clamped;
    const bool hardNoise = (kind == VAND_AUDIO_CLASS_DRUM) || (kind == VAND_AUDIO_CLASS_SAMPLE);

    PSG_setNoise(hardNoise ? PSG_NOISE_TYPE_WHITE : PSG_NOISE_TYPE_PERIODIC, hardNoise ? PSG_NOISE_FREQ_CLOCK8 : PSG_NOISE_FREQ_CLOCK4);
    PSG_setEnvelope(3, envelope);
}

static void applyEvent(const VandAudioEvent *event)
{
    ymKey(VAND_AUDIO_FM_BASS_CH, FALSE);
    if ((event->fm0_fnum > 0) && (event->fm0_level > 0))
    {
        ymSetFrequency(VAND_AUDIO_FM_BASS_CH, event->fm0_fnum, event->fm0_block);
        ymSetLevel(VAND_AUDIO_FM_BASS_CH, event->fm0_level);
        ymKey(VAND_AUDIO_FM_BASS_CH, TRUE);
    }

    ymKey(VAND_AUDIO_FM_LEAD_CH, FALSE);
    if ((event->fm1_fnum > 0) && (event->fm1_level > 0))
    {
        ymSetFrequency(VAND_AUDIO_FM_LEAD_CH, event->fm1_fnum, event->fm1_block);
        ymSetLevel(VAND_AUDIO_FM_LEAD_CH, event->fm1_level);
        ymKey(VAND_AUDIO_FM_LEAD_CH, TRUE);
    }

    psgSetNoiseLevel(event->psg_noise_level, event->kind);
}

void VandAudio_init(void)
{
    YM2612_writeReg(0, 0x22, 0x00);
    YM2612_writeReg(0, 0x27, 0x00);
    YM2612_writeReg(0, 0x2B, 0x00);
    ymInitVoice(VAND_AUDIO_FM_BASS_CH, TRUE);
    ymInitVoice(VAND_AUDIO_FM_LEAD_CH, FALSE);
    PSG_setEnvelope(3, PSG_ENVELOPE_MIN);
    audioReady = TRUE;
}

void VandAudio_start(VandAudioPlayer *player, const VandAudioEvent *events, u16 eventCount, bool loop)
{
    if (!audioReady) VandAudio_init();

    player->events = events;
    player->eventCount = eventCount;
    player->index = 0;
    player->wait = 0;
    player->loop = loop;
    player->playing = (events != NULL) && (eventCount > 0);
}

void VandAudio_stop(VandAudioPlayer *player)
{
    if (player != NULL)
    {
        player->playing = FALSE;
        player->events = NULL;
        player->eventCount = 0;
        player->index = 0;
        player->wait = 0;
    }

    ymKey(VAND_AUDIO_FM_BASS_CH, FALSE);
    ymKey(VAND_AUDIO_FM_LEAD_CH, FALSE);
    PSG_setEnvelope(3, PSG_ENVELOPE_MIN);
}

void VandAudio_update(VandAudioPlayer *player)
{
    const VandAudioEvent *event;

    if ((player == NULL) || !player->playing || (player->events == NULL)) return;

    if (player->wait > 0)
    {
        player->wait--;
        return;
    }

    if (player->index >= player->eventCount)
    {
        if (!player->loop)
        {
            VandAudio_stop(player);
            return;
        }
        player->index = 0;
    }

    event = &player->events[player->index++];
    applyEvent(event);
    player->wait = event->frames > 0 ? event->frames - 1 : 0;
}
