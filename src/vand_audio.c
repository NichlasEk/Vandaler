#include "vand_audio.h"

#define VAND_AUDIO_FM_BASS_CH 0
#define VAND_AUDIO_FM_LEAD_CH 1
#define VAND_AUDIO_FM_PAD_CH 2
#define VAND_AUDIO_FM_CHORD_CH 3
#define VAND_AUDIO_FM_COUNTER_CH 4
#define VAND_AUDIO_FM_CHANNELS 5
#define VAND_AUDIO_FM_KEY_ALL 0xF0
#define VAND_AUDIO_DAC_NONE 255
#define VAND_AUDIO_DAC_DEFAULT_RATE 8000
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
    const u8 carrierTl = 127 - (clamped * 7);
    const u8 part = ymPart(channel);
    const u8 slot = ymSlot(channel);

    YM2612_writeReg(part, 0x40 + slot, 112);
    YM2612_writeReg(part, 0x44 + slot, 112);
    YM2612_writeReg(part, 0x48 + slot, 112);
    YM2612_writeReg(part, 0x4C + slot, carrierTl);
}

static void ymInitVoice(u8 channel, bool bass)
{
    const u8 detuneMul[4] = {bass ? 0x01 : 0x02, bass ? 0x02 : 0x03, 0x01, 0x01};
    const u8 rates[4] = {0x1F, bass ? 0x18 : 0x12, bass ? 0x14 : 0x10, bass ? 0x16 : 0x14};
    const u8 release[4] = {0x0F, 0x0F, 0x0F, 0x0F};
    const u8 sustain[4] = {0x08, 0x0A, 0x0C, 0x08};
    const u8 offsets[4] = {0x00, 0x04, 0x08, 0x0C};
    const u8 part = ymPart(channel);
    const u8 slot = ymSlot(channel);
    u8 op;

    YM2612_writeReg(part, 0xB0 + slot, bass ? 0x32 : 0x31);
    YM2612_writeReg(part, 0xB4 + slot, 0xC0);

    for (op = 0; op < 4; op++)
    {
        const u8 reg = offsets[op] + slot;
        YM2612_writeReg(part, 0x30 + reg, detuneMul[op]);
        YM2612_writeReg(part, 0x50 + reg, rates[op]);
        YM2612_writeReg(part, 0x60 + reg, 0x05);
        YM2612_writeReg(part, 0x70 + reg, 0x02);
        YM2612_writeReg(part, 0x80 + reg, release[op]);
        YM2612_writeReg(part, 0x90 + reg, sustain[op]);
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
        const u8 sample = (u8)(128 + ((centered * dacLevel) / 15));
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
    ymInitVoice(VAND_AUDIO_FM_BASS_CH, TRUE);
    ymInitVoice(VAND_AUDIO_FM_LEAD_CH, FALSE);
    ymInitVoice(VAND_AUDIO_FM_PAD_CH, FALSE);
    ymInitVoice(VAND_AUDIO_FM_CHORD_CH, FALSE);
    ymInitVoice(VAND_AUDIO_FM_COUNTER_CH, FALSE);
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

    ymKey(VAND_AUDIO_FM_BASS_CH, FALSE);
    ymKey(VAND_AUDIO_FM_LEAD_CH, FALSE);
    ymKey(VAND_AUDIO_FM_PAD_CH, FALSE);
    fmCurrentFnum[0] = 0;
    fmCurrentFnum[1] = 0;
    fmCurrentFnum[2] = 0;
    fmCurrentBlock[0] = 0;
    fmCurrentBlock[1] = 0;
    fmCurrentBlock[2] = 0;
    fmCurrentLevel[0] = 0;
    fmCurrentLevel[1] = 0;
    fmCurrentLevel[2] = 0;
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
