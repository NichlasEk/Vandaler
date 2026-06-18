#include "vand_audio.h"

#define VAND_AUDIO_FM_BASS_CH 0
#define VAND_AUDIO_FM_LEAD_CH 1
#define VAND_AUDIO_FM_PAD_CH 2
#define VAND_AUDIO_FM_CHORD_CH 3
#define VAND_AUDIO_FM_COUNTER_CH 4
#define VAND_AUDIO_FM_CHANNELS 5
#define VAND_AUDIO_ROLE_BASS 0
#define VAND_AUDIO_ROLE_LEAD 1
#define VAND_AUDIO_ROLE_PAD 2
#define VAND_AUDIO_ROLE_CHORD 3
#define VAND_AUDIO_ROLE_COUNTER 4
#define VAND_AUDIO_FM_KEY_ALL 0xF0
#define VAND_AUDIO_DAC_NONE 255
#define VAND_AUDIO_DAC_DEFAULT_RATE 8000
#define VAND_AUDIO_DAC_BOOST 2
#define VAND_AUDIO_TICKS_PER_SECOND 60

static bool audioReady = FALSE;
static const u8 * const *dacSamples = NULL;
static const u16 *dacLengths = NULL;
static const u16 *dacRates = NULL;
static u16 dacCount = 0;
static const u8 *dacActive = NULL;
static u16 dacLength = 0;
static u16 dacPos = 0;
static u16 dacRate = 0;
static u16 dacRateRemainder = 0;
static u8 dacLevel = 0;
static u16 fmCurrentFnum[VAND_AUDIO_FM_CHANNELS] = {0, 0, 0, 0, 0};
static u8 fmCurrentBlock[VAND_AUDIO_FM_CHANNELS] = {0, 0, 0, 0, 0};
static u8 fmCurrentLevel[VAND_AUDIO_FM_CHANNELS] = {0, 0, 0, 0, 0};
static const u8 fmRoleForChannel[VAND_AUDIO_FM_CHANNELS] = {
    VAND_AUDIO_ROLE_BASS,
    VAND_AUDIO_ROLE_LEAD,
    VAND_AUDIO_ROLE_PAD,
    VAND_AUDIO_ROLE_CHORD,
    VAND_AUDIO_ROLE_COUNTER
};
static const u8 fmAlgorithm[VAND_AUDIO_FM_CHANNELS] = {0x20, 0x3A, 0x3B, 0x3B, 0x3A};
static const u8 fmFeedbackStereo[VAND_AUDIO_FM_CHANNELS] = {0xF0, 0xC0, 0xC0, 0xC0, 0xC0};
static const u8 fmDetuneMul[VAND_AUDIO_FM_CHANNELS][4] = {
    {0x51, 0x50, 0x02, 0x00},
    {0x31, 0x31, 0x35, 0x31},
    {0x32, 0x31, 0x42, 0x32},
    {0x32, 0x31, 0x42, 0x32},
    {0x31, 0x31, 0x35, 0x31}
};
static const u8 fmAttack[VAND_AUDIO_FM_CHANNELS][4] = {
    {0x1F, 0x1F, 0x12, 0x1F},
    {0x8E, 0x8D, 0x8E, 0x53},
    {0x9F, 0x9F, 0x1F, 0x9F},
    {0x9F, 0x9F, 0x1F, 0x9F},
    {0x8E, 0x8D, 0x8E, 0x53}
};
static const u8 fmDecay[VAND_AUDIO_FM_CHANNELS][4] = {
    {0x05, 0x0D, 0x0B, 0x09},
    {0x0E, 0x0E, 0x0E, 0x03},
    {0x03, 0x03, 0x0C, 0x03},
    {0x03, 0x03, 0x0C, 0x03},
    {0x0E, 0x0E, 0x0E, 0x03}
};
static const u8 fmSustainRate[VAND_AUDIO_FM_CHANNELS][4] = {
    {0x00, 0x00, 0x00, 0x00},
    {0x00, 0x00, 0x00, 0x00},
    {0x00, 0x00, 0x0D, 0x00},
    {0x00, 0x00, 0x0D, 0x00},
    {0x00, 0x00, 0x00, 0x00}
};
static const u8 fmRelease[VAND_AUDIO_FM_CHANNELS][4] = {
    {0x0E, 0x0E, 0x0E, 0x0E},
    {0x0F, 0x0F, 0x0F, 0x0F},
    {0x09, 0x08, 0x07, 0x07},
    {0x09, 0x08, 0x07, 0x07},
    {0x0F, 0x0F, 0x0F, 0x0F}
};
static const u8 fmSustainLevel[VAND_AUDIO_FM_CHANNELS][4] = {
    {0x07, 0x06, 0x08, 0x0F},
    {0x01, 0x01, 0x0F, 0x00},
    {0x0F, 0x0F, 0x07, 0x0F},
    {0x0F, 0x0F, 0x07, 0x0F},
    {0x01, 0x01, 0x0F, 0x00}
};
static const u8 fmBaseTl[VAND_AUDIO_FM_CHANNELS][4] = {
    {32, 23, 23, 0},
    {15, 28, 30, 11},
    {48, 43, 42, 32},
    {44, 39, 38, 28},
    {19, 32, 34, 15}
};
static const u8 fmLevelScale[VAND_AUDIO_FM_CHANNELS][4] = {
    {0, 0, 1, 6},
    {1, 1, 1, 6},
    {1, 1, 2, 3},
    {1, 1, 2, 4},
    {1, 1, 1, 5}
};

static u8 ymPart(u8 channel)
{
    return channel >= 3 ? 1 : 0;
}

static u8 ymSlot(u8 channel)
{
    return channel >= 3 ? channel - 3 : channel;
}

static u8 ymKeyChannel(u8 channel)
{
    return channel >= 3 ? channel + 1 : channel;
}

static void ymKey(u8 channel, bool on)
{
    YM2612_writeReg(0, 0x28, (on ? VAND_AUDIO_FM_KEY_ALL : 0x00) | ymKeyChannel(channel));
}

static void ymSetFrequency(u8 channel, u16 fnum, u8 block)
{
    const u8 part = ymPart(channel);
    const u8 slot = ymSlot(channel);
    YM2612_writeReg(part, 0xA4 + slot, ((block & 0x07) << 3) | ((fnum >> 8) & 0x07));
    YM2612_writeReg(part, 0xA0 + slot, fnum & 0xFF);
}

static void ymSetLevel(u8 channel, u8 level)
{
    const u8 clamped = level > 15 ? 15 : level;
    const u8 part = ymPart(channel);
    const u8 slot = ymSlot(channel);
    const u8 role = fmRoleForChannel[channel];
    const u8 offsets[4] = {0x00, 0x04, 0x08, 0x0C};
    u8 op;

    for (op = 0; op < 4; op++)
    {
        const u8 reg = offsets[op] + slot;
        const u8 baseTl = fmBaseTl[role][op];
        const u8 scale = fmLevelScale[role][op];
        const u8 tl = clamped == 0 ? 127 : (baseTl > clamped * scale ? baseTl - clamped * scale : 0);
        YM2612_writeReg(part, 0x40 + reg, tl);
    }
}

static void ymInitVoice(u8 channel)
{
    const u8 offsets[4] = {0x00, 0x04, 0x08, 0x0C};
    const u8 part = ymPart(channel);
    const u8 slot = ymSlot(channel);
    const u8 role = fmRoleForChannel[channel];
    u8 op;

    YM2612_writeReg(part, 0xB0 + slot, fmAlgorithm[role]);
    YM2612_writeReg(part, 0xB4 + slot, fmFeedbackStereo[role]);

    for (op = 0; op < 4; op++)
    {
        const u8 reg = offsets[op] + slot;
        YM2612_writeReg(part, 0x30 + reg, fmDetuneMul[role][op]);
        YM2612_writeReg(part, 0x50 + reg, fmAttack[role][op]);
        YM2612_writeReg(part, 0x60 + reg, fmDecay[role][op]);
        YM2612_writeReg(part, 0x70 + reg, fmSustainRate[role][op]);
        YM2612_writeReg(part, 0x80 + reg, ((fmSustainLevel[role][op] & 0x0F) << 4) | (fmRelease[role][op] & 0x0F));
        YM2612_writeReg(part, 0x90 + reg, 0x00);
    }

    ymSetLevel(channel, 0);
}

static void ymApplyChannel(u8 channel, u16 fnum, u8 block, u8 level)
{
    const bool wantOn = (fnum > 0) && (level > 0);
    const bool wasOn = fmCurrentLevel[channel] > 0;
    const bool pitchChanged = (fmCurrentFnum[channel] != fnum) || (fmCurrentBlock[channel] != block);
    const bool levelChanged = fmCurrentLevel[channel] != level;

    if (!wantOn)
    {
        if (wasOn) ymKey(channel, FALSE);
        if (wasOn) ymSetLevel(channel, 0);
        fmCurrentFnum[channel] = 0;
        fmCurrentBlock[channel] = 0;
        fmCurrentLevel[channel] = 0;
        return;
    }

    if (!wasOn || pitchChanged)
    {
        if (wasOn) ymKey(channel, FALSE);
        ymSetFrequency(channel, fnum, block);
        ymSetLevel(channel, level);
        ymKey(channel, TRUE);
    }
    else if (levelChanged)
    {
        ymSetLevel(channel, level);
    }

    fmCurrentFnum[channel] = fnum;
    fmCurrentBlock[channel] = block;
    fmCurrentLevel[channel] = level;
}

static void psgSetNoiseLevel(u8 level, u8 kind)
{
    const u8 clamped = level > 15 ? 15 : level;
    const u8 envelope = clamped == 0 ? PSG_ENVELOPE_MIN : 15 - clamped;
    const bool hardNoise = (kind == VAND_AUDIO_CLASS_DRUM) || (kind == VAND_AUDIO_CLASS_SAMPLE);

    PSG_setNoise(hardNoise ? PSG_NOISE_TYPE_WHITE : PSG_NOISE_TYPE_PERIODIC, hardNoise ? PSG_NOISE_FREQ_CLOCK8 : PSG_NOISE_FREQ_CLOCK4);
    PSG_setEnvelope(3, envelope);
}

static void psgSetToneLevel(u8 channel, u16 tone, u8 level)
{
    const u8 clamped = level > 15 ? 15 : level;

    if (channel > 2) return;
    if ((tone > 0) && (clamped > 0))
    {
        PSG_setTone(channel, tone);
        PSG_setEnvelope(channel, 15 - clamped);
        return;
    }

    PSG_setEnvelope(channel, PSG_ENVELOPE_MIN);
}

static void dacStop(void)
{
    dacActive = NULL;
    dacLength = 0;
    dacPos = 0;
    dacRate = 0;
    dacRateRemainder = 0;
    dacLevel = 0;
    YM2612_writeReg(0, 0x2A, 0x80);
}

static void dacStart(u8 chunk, u8 level)
{
    if ((chunk == VAND_AUDIO_DAC_NONE) || (level == 0) || (dacSamples == NULL) || (dacLengths == NULL) || (chunk >= dacCount)) return;

    dacActive = dacSamples[chunk];
    dacLength = dacLengths[chunk];
    dacPos = 0;
    dacRate = ((dacRates != NULL) && (dacRates[chunk] > 0)) ? dacRates[chunk] : VAND_AUDIO_DAC_DEFAULT_RATE;
    dacRateRemainder = 0;
    dacLevel = level > 15 ? 15 : level;
    if ((dacActive == NULL) || (dacLength == 0)) dacStop();
}

static void dacPump(void)
{
    u16 samplesThisTick;
    u16 i;

    if (dacActive == NULL) return;

    dacRateRemainder += dacRate;
    samplesThisTick = dacRateRemainder / VAND_AUDIO_TICKS_PER_SECOND;
    dacRateRemainder %= VAND_AUDIO_TICKS_PER_SECOND;

    for (i = 0; i < samplesThisTick; i++)
    {
        const s16 centered = (s16)dacActive[dacPos++] - 128;
        s16 scaled = 128 + ((centered * dacLevel * VAND_AUDIO_DAC_BOOST) / 15);
        if (scaled < 0) scaled = 0;
        if (scaled > 255) scaled = 255;
        const u8 sample = (u8)scaled;
        YM2612_writeReg(0, 0x2A, sample);
        if (dacPos >= dacLength)
        {
            dacStop();
            return;
        }
    }
}

static void applyEvent(const VandAudioEvent *event)
{
    ymApplyChannel(VAND_AUDIO_FM_BASS_CH, event->fm0_fnum, event->fm0_block, event->fm0_level);
    ymApplyChannel(VAND_AUDIO_FM_LEAD_CH, event->fm1_fnum, event->fm1_block, event->fm1_level);
    ymApplyChannel(VAND_AUDIO_FM_PAD_CH, event->fm2_fnum, event->fm2_block, event->fm2_level);
    ymApplyChannel(VAND_AUDIO_FM_CHORD_CH, event->fm3_fnum, event->fm3_block, event->fm3_level);
    ymApplyChannel(VAND_AUDIO_FM_COUNTER_CH, event->fm4_fnum, event->fm4_block, event->fm4_level);

    psgSetToneLevel(0, event->psg0_tone, event->psg0_level);
    psgSetToneLevel(1, event->psg1_tone, event->psg1_level);
    psgSetToneLevel(2, event->psg2_tone, event->psg2_level);
    psgSetNoiseLevel(event->psg_noise_level, event->kind);
    dacStart(event->dac_chunk, event->dac_level);
}

void VandAudio_init(void)
{
    YM2612_writeReg(0, 0x22, 0x00);
    YM2612_writeReg(0, 0x27, 0x00);
    YM2612_writeReg(0, 0x2B, 0x80);
    YM2612_writeReg(0, 0x2A, 0x80);
    ymInitVoice(VAND_AUDIO_FM_BASS_CH);
    ymInitVoice(VAND_AUDIO_FM_LEAD_CH);
    ymInitVoice(VAND_AUDIO_FM_PAD_CH);
    ymInitVoice(VAND_AUDIO_FM_CHORD_CH);
    ymInitVoice(VAND_AUDIO_FM_COUNTER_CH);
    for (u8 i = 0; i < VAND_AUDIO_FM_CHANNELS; i++)
    {
        fmCurrentFnum[i] = 0;
        fmCurrentBlock[i] = 0;
        fmCurrentLevel[i] = 0;
    }
    PSG_setEnvelope(0, PSG_ENVELOPE_MIN);
    PSG_setEnvelope(1, PSG_ENVELOPE_MIN);
    PSG_setEnvelope(2, PSG_ENVELOPE_MIN);
    PSG_setEnvelope(3, PSG_ENVELOPE_MIN);
    audioReady = TRUE;
}

void VandAudio_setDacBank(const u8 * const *samples, const u16 *lengths, const u16 *rates, u16 count)
{
    dacSamples = samples;
    dacLengths = lengths;
    dacRates = rates;
    dacCount = count;
    dacStop();
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

    for (u8 i = 0; i < VAND_AUDIO_FM_CHANNELS; i++)
    {
        ymKey(i, FALSE);
        fmCurrentFnum[i] = 0;
        fmCurrentBlock[i] = 0;
        fmCurrentLevel[i] = 0;
    }
    PSG_setEnvelope(0, PSG_ENVELOPE_MIN);
    PSG_setEnvelope(1, PSG_ENVELOPE_MIN);
    PSG_setEnvelope(2, PSG_ENVELOPE_MIN);
    PSG_setEnvelope(3, PSG_ENVELOPE_MIN);
    dacStop();
}

void VandAudio_update(VandAudioPlayer *player)
{
    const VandAudioEvent *event;

    dacPump();

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
