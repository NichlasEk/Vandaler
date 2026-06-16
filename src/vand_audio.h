#ifndef VAND_AUDIO_H
#define VAND_AUDIO_H

#include <genesis.h>

typedef enum
{
    VAND_AUDIO_CLASS_SILENCE = 0,
    VAND_AUDIO_CLASS_AMBIENCE = 1,
    VAND_AUDIO_CLASS_NOISE = 2,
    VAND_AUDIO_CLASS_SAMPLE = 3,
    VAND_AUDIO_CLASS_DRUM = 4,
    VAND_AUDIO_CLASS_BASS = 5,
    VAND_AUDIO_CLASS_TONAL = 6
} VandAudioClass;

typedef struct
{
    u16 frames;
    u16 fm0_fnum;
    u8 fm0_block;
    u8 fm0_level;
    u16 fm1_fnum;
    u8 fm1_block;
    u8 fm1_level;
    u16 fm2_fnum;
    u8 fm2_block;
    u8 fm2_level;
    u16 fm3_fnum;
    u8 fm3_block;
    u8 fm3_level;
    u16 fm4_fnum;
    u8 fm4_block;
    u8 fm4_level;
    u8 psg_noise_level;
    u8 kind;
    u8 dac_chunk;
    u8 dac_level;
} VandAudioEvent;

typedef struct
{
    const VandAudioEvent *events;
    u16 eventCount;
    u16 index;
    u16 wait;
    bool loop;
    bool playing;
} VandAudioPlayer;

void VandAudio_init(void);
void VandAudio_setDacBank(const u8 * const *samples, const u16 *lengths, const u16 *rates, u16 count);
void VandAudio_start(VandAudioPlayer *player, const VandAudioEvent *events, u16 eventCount, bool loop);
void VandAudio_stop(VandAudioPlayer *player);
void VandAudio_update(VandAudioPlayer *player);

#endif
