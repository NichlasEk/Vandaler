#include <genesis.h>
#include <psg.h>
#include "resources.h"

#define SCREEN_TILES_W 40
#define SCREEN_TILES_H 28
#define HUD_TILES_H 4
#define FLOOR_Y 23
#define MAX_BUILDINGS 5
#define MAX_MONSTERS 3
#define MAX_ENEMIES 3
#define MAX_TANKS 2
#define MAX_HELIS 2
#define MAX_SHOTS 10
#define MAX_PEOPLE 8
#define MAX_EXPLOSIONS 4
#define MAX_DAMAGE_MARKS 16
#define MAX_WINDOW_PEOPLE 4
#define BUILDING_CRACK_STEP_FRAMES 8
#define BUILDING_COLLAPSE_STEP_FRAMES 6
#define PLAYER_STUN_FRAMES 90
#define PLAYER_MAX_HEALTH 8
#define PLAYER_W 48
#define PLAYER_H 56
#define PLAYER_GROUND_Y ((FLOOR_Y - 7) * 8)
#define PLAYER_SPRITE_Y_OFFSET 9
#define PLAYER_CLIMB_SPRITE_X_OFFSET 9
#define GROUND_SPRITE_Y_OFFSET 8
#define PLAYER_SPRITE_FRAMES 14
#define POSE_IDLE 0
#define POSE_CLIMB_UP 1
#define POSE_PUNCH 2
#define POSE_PUNCH_UP 3
#define POSE_PUNCH_DIAG_UP 4
#define POSE_PUNCH_DIAG_DOWN 5
#define POSE_CLIMB_DOWN 6
#define POSE_CLIMB_PUNCH_IN 7
#define POSE_CLIMB_PUNCH_UP 8
#define POSE_CLIMB_PUNCH_DIAG_UP 9
#define POSE_CLIMB_PUNCH_DIAG_DOWN 10
#define POSE_CLIMB_PUNCH_OUT 11
#define POSE_WALK_A 12
#define POSE_WALK_B 13
#define POSE_STUNNED POSE_PUNCH_DIAG_DOWN
#define PLAYER_HURT_X 14
#define PLAYER_HURT_Y 10
#define PLAYER_HURT_W 20
#define PLAYER_HURT_H 38
#define PLAYER_JUMP_VY -7
#define PLAYER_BOUNCE_VY -4
#define PLAYER_GRAVITY_STEP 2
#define PLAYER_BOUNCE_GRAVITY_STEP 6
#define PLAYER_MAX_FALL 6
#define PLAYER_BOUNCE_MAX_FALL 2
#define ENEMY_HURT_X 7
#define ENEMY_HURT_Y 5
#define ENEMY_HURT_W 10
#define ENEMY_HURT_H 8
#define SHOT_HURT_X 2
#define SHOT_HURT_Y 2
#define SHOT_HURT_W 4
#define SHOT_HURT_H 4
#define PERSON_EAT_X -6
#define PERSON_EAT_Y -4
#define PERSON_EAT_W 24
#define PERSON_EAT_H 28
#define PLAYER_BITE_X 10
#define PLAYER_BITE_Y 18
#define PLAYER_BITE_W 46
#define PLAYER_BITE_H 38

#define TILE_SKY     (TILE_USER_INDEX + 0)
#define TILE_DARK    (TILE_USER_INDEX + 1)
#define TILE_ROAD    (TILE_USER_INDEX + 2)
#define TILE_LINE    (TILE_USER_INDEX + 3)
#define TILE_GREEN   (TILE_USER_INDEX + 4)
#define TILE_WIN_ON  (TILE_USER_INDEX + 5)
#define TILE_WIN_OFF (TILE_USER_INDEX + 6)
#define TILE_CRACK   (TILE_USER_INDEX + 7)
#define TILE_CRACK_SMALL (TILE_USER_INDEX + 8)
#define TILE_CRACK_MID   (TILE_USER_INDEX + 9)
#define TILE_CRACK_LARGE (TILE_USER_INDEX + 10)
#define TILE_DUST_1      (TILE_USER_INDEX + 11)
#define TILE_DUST_2      (TILE_USER_INDEX + 12)
#define TILE_DUST_3      (TILE_USER_INDEX + 13)
#define TILE_RED         (TILE_USER_INDEX + 14)
#define TILE_WHITE       (TILE_USER_INDEX + 15)
#define TILE_YELLOW      (TILE_USER_INDEX + 16)
#define TILE_A_RING      (TILE_USER_INDEX + 17)
#define TILE_A_UML       (TILE_USER_INDEX + 18)
#define TILE_O_UML       (TILE_USER_INDEX + 19)

typedef enum
{
    STATE_TITLE,
    STATE_SELECT,
    STATE_INTRO,
    STATE_PLAY,
    STATE_CLEAR,
    STATE_GAME_OVER
} GameState;

typedef struct
{
    const char *name;
    const char *species;
    u8 pal;
    u8 accentPal;
} MonsterDef;

typedef struct
{
    s16 x;
    s16 y;
    s16 w;
    s16 h;
    s8 xDir;
    s8 yDir;
    u8 pose;
} AttackBox;

typedef struct
{
    s16 x;
    s16 y;
    s16 vy;
    s16 dir;
    u8 monster;
    u8 punching;
    u8 punchTimer;
    u8 attackPose;
    u8 climbPose;
    u8 landTimer;
    u8 stunTimer;
    u8 gravityTimer;
    AttackBox attack;
    bool walking;
    bool grounded;
} Player;

typedef struct
{
    s16 x;
    u8 y;
    u8 w;
    u8 h;
    u8 damage;
    u8 damageX[MAX_DAMAGE_MARKS];
    u8 damageY[MAX_DAMAGE_MARKS];
    u8 personX[MAX_WINDOW_PEOPLE];
    u8 personY[MAX_WINDOW_PEOPLE];
    bool personAlive[MAX_WINDOW_PEOPLE];
    bool personEdible[MAX_WINDOW_PEOPLE];
    u8 colorPal;
    u8 crackTimer;
    u8 crackRows;
    u8 collapseTimer;
    u8 collapseRows;
    bool cracking;
    bool collapsing;
    bool alive;
} Building;

typedef struct
{
    s16 x;
    s16 y;
    s16 speed;
    u8 stepTimer;
    bool active;
    Sprite *sprite;
} Enemy;

typedef struct
{
    s16 x;
    s16 y;
    s16 speed;
    u8 stepTimer;
    u8 cooldown;
    bool active;
    Sprite *sprite;
} Tank;

typedef struct
{
    s16 x;
    s16 y;
    s16 speed;
    u8 stepTimer;
    u8 cooldown;
    u8 burst;
    u8 burstDelay;
    bool active;
    Sprite *sprite;
} Helicopter;

typedef struct
{
    s16 x;
    s16 y;
    s16 dx;
    s16 dy;
    u8 stepTimer;
    bool active;
    Sprite *sprite;
} Shot;

typedef struct
{
    s16 x;
    s16 y;
    s16 vy;
    s16 speed;
    bool falling;
    bool edible;
    bool active;
    Sprite *sprite;
} Person;

typedef struct
{
    s16 x;
    s16 y;
    u8 timer;
    bool active;
    Sprite *sprite;
} Explosion;

typedef struct
{
    bool active;
    s16 snapX;
    s8 attackDir;
    u8 building;
} ClimbContact;

typedef struct
{
    bool active;
    s16 y;
    u8 building;
} RoofContact;

static const u32 tileSky[8]     = {0x11111111,0x111C1111,0x11111111,0x111111C1,0x11111111,0x11C11111,0x11111111,0x11111C11};
static const u32 tileDark[8]    = {0xBBBBBBBB,0xB2B2B2B2,0xBBBBBBBB,0x2B2B2B2B,0xBBBBBBBB,0xB2B2B2B2,0xBBBBBBBB,0x2B2B2B2B};
static const u32 tileRoad[8]    = {0x3B333333,0x333333B3,0x333B3333,0x33333333,0xB333333B,0x33333B33,0x33333333,0x33B33333};
static const u32 tileLine[8]    = {0x33333333,0x33333333,0x33333333,0x99999999,0x99999999,0x33333333,0x33333333,0x33333333};
static const u32 tileGreen[8]   = {0x00044000,0x004C4400,0x04444440,0x0C4444C0,0x00444400,0x000A0000,0x000A0000,0x000A0000};
static const u32 tileWinOn[8]   = {0x00000000,0x02222200,0x02666200,0x026C6200,0x02666200,0x02222200,0x00000000,0x00000000};
static const u32 tileWinOff[8]  = {0x00000000,0x02222200,0x02222200,0x02000200,0x02000200,0x02222200,0x00000000,0x00000000};
static const u32 tileCrack[8]   = {0x00080000,0x00028000,0x00888000,0x08820000,0x00288000,0x00028800,0x00008000,0x00000000};
static const u32 tileCrackSmall[8] = {0x00000000,0x00080000,0x00088000,0x00028000,0x00008800,0x00000800,0x00000000,0x00000000};
static const u32 tileCrackMid[8]   = {0x00080000,0x00088000,0x00828000,0x08880000,0x00288000,0x00028800,0x00008000,0x00000000};
static const u32 tileCrackLarge[8] = {0x80008000,0x88088000,0x02880000,0x08888200,0x00288000,0x08228800,0x88008000,0x80000080};
static const u32 tileDust1[8]   = {0x00000000,0x00000000,0x00000000,0x90909090,0x09999990,0x999F9999,0x09F99990,0x00000000};
static const u32 tileDust2[8]   = {0x00000000,0x00000000,0x09090000,0x99999990,0x9F9F9F99,0x99999999,0x09999900,0x00000000};
static const u32 tileDust3[8]   = {0x00000000,0x09009000,0x99999909,0x9F999999,0x999F9F99,0x09999990,0x00099000,0x00000000};
static const u32 tileRed[8]     = {0x88888888,0x8D8D8D8D,0xDDDDDDDD,0x88888888,0x8D8D8D8D,0xDDDDDDDD,0x88888888,0x88888888};
static const u32 tileWhite[8]   = {0x99999999,0x9F9F9F9F,0xFFFFFFFF,0x99999999,0x99999999,0xFFFFFFFF,0x9F9F9F9F,0x99999999};
static const u32 tileYellow[8]  = {0xDDDDDDDD,0xD77777DD,0xD77777DD,0xD7777DDD,0xD77777DD,0xD77777DD,0xDD7777DD,0xDDDDDDDD};
static const u32 tileARing[8]   = {0x000FF000,0x00F00F00,0x000FF000,0x00F00F00,0x0F0000F0,0x0FFFFFF0,0x0F0000F0,0x0F0000F0};
static const u32 tileAUml[8]    = {0x00F00F00,0x00000000,0x00F00F00,0x0F0000F0,0x0FFFFFF0,0x0F0000F0,0x0F0000F0,0x00000000};
static const u32 tileOUml[8]    = {0x00F00F00,0x00000000,0x00FFFF00,0x0F0000F0,0x0F0000F0,0x0F0000F0,0x00FFFF00,0x00000000};

static const u16 palette0[16] =
{
    RGB3_3_3_TO_VDPCOLOR(0,0,0),
    RGB3_3_3_TO_VDPCOLOR(0,1,5),
    RGB3_3_3_TO_VDPCOLOR(0,0,0),
    RGB3_3_3_TO_VDPCOLOR(3,3,4),
    RGB3_3_3_TO_VDPCOLOR(0,6,1),
    RGB3_3_3_TO_VDPCOLOR(0,3,1),
    RGB3_3_3_TO_VDPCOLOR(3,5,7),
    RGB3_3_3_TO_VDPCOLOR(5,3,1),
    RGB3_3_3_TO_VDPCOLOR(7,1,0),
    RGB3_3_3_TO_VDPCOLOR(7,7,7),
    RGB3_3_3_TO_VDPCOLOR(3,1,0),
    RGB3_3_3_TO_VDPCOLOR(1,1,3),
    RGB3_3_3_TO_VDPCOLOR(0,0,3),
    RGB3_3_3_TO_VDPCOLOR(7,5,2),
    RGB3_3_3_TO_VDPCOLOR(0,0,0),
    RGB3_3_3_TO_VDPCOLOR(7,7,7)
};

static const u16 paletteApe[16] =
{
    RGB3_3_3_TO_VDPCOLOR(0,0,0), RGB3_3_3_TO_VDPCOLOR(2,1,0), RGB3_3_3_TO_VDPCOLOR(4,2,0), RGB3_3_3_TO_VDPCOLOR(6,4,1),
    RGB3_3_3_TO_VDPCOLOR(7,5,2), RGB3_3_3_TO_VDPCOLOR(7,7,7), RGB3_3_3_TO_VDPCOLOR(7,1,0), RGB3_3_3_TO_VDPCOLOR(0,0,0),
    RGB3_3_3_TO_VDPCOLOR(0,0,0), RGB3_3_3_TO_VDPCOLOR(0,0,0), RGB3_3_3_TO_VDPCOLOR(0,0,0), RGB3_3_3_TO_VDPCOLOR(0,0,0),
    RGB3_3_3_TO_VDPCOLOR(0,0,0), RGB3_3_3_TO_VDPCOLOR(0,0,0), RGB3_3_3_TO_VDPCOLOR(0,0,0), RGB3_3_3_TO_VDPCOLOR(0,0,0)
};

static const u16 paletteWolf[16] =
{
    RGB3_3_3_TO_VDPCOLOR(0,0,0), RGB3_3_3_TO_VDPCOLOR(1,1,2), RGB3_3_3_TO_VDPCOLOR(3,3,5), RGB3_3_3_TO_VDPCOLOR(5,5,7),
    RGB3_3_3_TO_VDPCOLOR(7,6,2), RGB3_3_3_TO_VDPCOLOR(7,7,7), RGB3_3_3_TO_VDPCOLOR(7,1,0), RGB3_3_3_TO_VDPCOLOR(0,0,0),
    RGB3_3_3_TO_VDPCOLOR(0,0,0), RGB3_3_3_TO_VDPCOLOR(0,0,0), RGB3_3_3_TO_VDPCOLOR(0,0,0), RGB3_3_3_TO_VDPCOLOR(0,0,0),
    RGB3_3_3_TO_VDPCOLOR(0,0,0), RGB3_3_3_TO_VDPCOLOR(0,0,0), RGB3_3_3_TO_VDPCOLOR(0,0,0), RGB3_3_3_TO_VDPCOLOR(0,0,0)
};

static const u16 paletteLizard[16] =
{
    RGB3_3_3_TO_VDPCOLOR(0,0,0), RGB3_3_3_TO_VDPCOLOR(0,2,0), RGB3_3_3_TO_VDPCOLOR(0,5,1), RGB3_3_3_TO_VDPCOLOR(1,7,2),
    RGB3_3_3_TO_VDPCOLOR(7,7,0), RGB3_3_3_TO_VDPCOLOR(7,7,7), RGB3_3_3_TO_VDPCOLOR(7,1,0), RGB3_3_3_TO_VDPCOLOR(0,0,0),
    RGB3_3_3_TO_VDPCOLOR(0,0,0), RGB3_3_3_TO_VDPCOLOR(0,0,0), RGB3_3_3_TO_VDPCOLOR(0,0,0), RGB3_3_3_TO_VDPCOLOR(0,0,0),
    RGB3_3_3_TO_VDPCOLOR(0,0,0), RGB3_3_3_TO_VDPCOLOR(0,0,0), RGB3_3_3_TO_VDPCOLOR(0,0,0), RGB3_3_3_TO_VDPCOLOR(0,0,0)
};

static const u16 paletteSelectOverlay[16] =
{
    RGB3_3_3_TO_VDPCOLOR(0,0,0), RGB3_3_3_TO_VDPCOLOR(0,0,0), RGB3_3_3_TO_VDPCOLOR(0,0,0), RGB3_3_3_TO_VDPCOLOR(0,0,0),
    RGB3_3_3_TO_VDPCOLOR(0,0,0), RGB3_3_3_TO_VDPCOLOR(0,0,0), RGB3_3_3_TO_VDPCOLOR(0,0,0), RGB3_3_3_TO_VDPCOLOR(0,0,0),
    RGB3_3_3_TO_VDPCOLOR(0,0,0), RGB3_3_3_TO_VDPCOLOR(0,0,0), RGB3_3_3_TO_VDPCOLOR(0,0,0), RGB3_3_3_TO_VDPCOLOR(0,0,0),
    RGB3_3_3_TO_VDPCOLOR(0,0,0), RGB3_3_3_TO_VDPCOLOR(0,0,0), RGB3_3_3_TO_VDPCOLOR(0,0,0), RGB3_3_3_TO_VDPCOLOR(7,6,0)
};

static const MonsterDef monsters[MAX_MONSTERS] =
{
    {"JONNY", "APA", PAL1, PAL0},
    {"CONNY", "VARG", PAL2, PAL0},
    {"BETTAN", "ODLA", PAL3, PAL0}
};

static const char *cities[] =
{
    "MALMÖ",
    "LINKÖPING",
    "ÖREBRO",
    "VÄSTERÅS",
    "STOCKHOLM",
    "UPPSALA",
    "GÄVLE",
    "SUNDSVALL",
    "UMEÅ",
    "LULEÅ"
};

static GameState state = STATE_TITLE;
static Player player;
static Sprite *playerSprite = NULL;
static Building buildings[MAX_BUILDINGS];
static Enemy enemies[MAX_ENEMIES];
static Tank tanks[MAX_TANKS];
static Helicopter helis[MAX_HELIS];
static Shot shots[MAX_SHOTS];
static Person people[MAX_PEOPLE];
static Explosion explosions[MAX_EXPLOSIONS];
static u8 selectedMonster = 0;
static u8 currentCity = 0;
static u8 playerHealth = PLAYER_MAX_HEALTH;
static u8 invulnerableTimer = 0;
static u16 frame = 0;
static u16 score = 0;
static u16 previousJoy = 0;
static bool hudFrameReady = FALSE;
static u16 hudScore = 0xFFFF;
static u8 hudHealth = 0xFF;
static u8 hudCity = 0xFF;
static u8 selectOverlayMonster = 0xFF;
static bool selectOverlayVisible = FALSE;

static ClimbContact getClimbContact(void);
static RoofContact getRoofContact(void);
static void loadTiles(void);
static void hidePlayerSprite(void);
static void hideThreatSprites(void);
static void resetSpriteEngineState(void);

static void loadGameVideo(void)
{
    PAL_setPalette(PAL0, palette0, DMA);
    PAL_setPalette(PAL1, paletteApe, DMA);
    PAL_setPalette(PAL2, paletteWolf, DMA);
    PAL_setPalette(PAL3, paletteLizard, DMA);
    loadTiles();
}

static void drawFullscreenImage(const Image *image)
{
    hidePlayerSprite();
    hideThreatSprites();
    VDP_clearPlane(BG_A, TRUE);
    VDP_clearPlane(BG_B, TRUE);
    PAL_setPalette(PAL0, image->palette->data, DMA);
    VDP_drawImageEx(BG_B, image, TILE_ATTR_FULL(PAL0, FALSE, FALSE, FALSE, TILE_USER_INDEX), 0, 0, FALSE, TRUE);
}

static u16 attr(u8 pal, u16 tile)
{
    return TILE_ATTR_FULL(pal, FALSE, FALSE, FALSE, tile);
}

static u16 svCharTile(const u8 **text)
{
    const u8 c = **text;

    if (c == 0) return 0;

    if (c == 0xC3)
    {
        const u8 next = *(*text + 1);
        *text += 2;

        switch (next)
        {
            case 0x85: // Å
            case 0xA5: // å
                return TILE_A_RING;

            case 0x84: // Ä
            case 0xA4: // ä
                return TILE_A_UML;

            case 0x96: // Ö
            case 0xB6: // ö
                return TILE_O_UML;

            default:
                return TILE_FONT_INDEX;
        }
    }

    (*text)++;
    if (c < 32 || c > 127) return TILE_FONT_INDEX;

    return TILE_FONT_INDEX + (c - 32);
}

static void drawTextSv(const char *text, u16 x, u16 y)
{
    const u8 *cursor = (const u8 *)text;
    u16 tx = x;

    while (*cursor != 0 && tx < SCREEN_TILES_W)
    {
        const u16 tile = svCharTile(&cursor);
        VDP_setTileMapXY(BG_A, attr(PAL0, tile), tx, y);
        tx++;
    }
}

static void drawAsciiTextBg(const char *text, u16 x, u16 y)
{
    u16 tx = x;

    while (*text != 0 && tx < SCREEN_TILES_W)
    {
        const u8 c = *text++;
        const u16 tile = (c < 32 || c > 127) ? TILE_FONT_INDEX : (TILE_FONT_INDEX + (c - 32));
        VDP_setTileMapXY(BG_B, c == ' ' ? attr(PAL0, TILE_DARK) : attr(PAL0, tile), tx, y);
        tx++;
    }
}

static void drawTextSvBg(const char *text, u16 x, u16 y)
{
    const u8 *cursor = (const u8 *)text;
    u16 tx = x;

    while (*cursor != 0 && tx < SCREEN_TILES_W)
    {
        const u16 tile = svCharTile(&cursor);
        VDP_setTileMapXY(BG_B, attr(PAL0, tile), tx, y);
        tx++;
    }
}

static void loadTiles(void)
{
    VDP_loadTileData(tileSky, TILE_SKY, 1, DMA);
    VDP_loadTileData(tileDark, TILE_DARK, 1, DMA);
    VDP_loadTileData(tileRoad, TILE_ROAD, 1, DMA);
    VDP_loadTileData(tileLine, TILE_LINE, 1, DMA);
    VDP_loadTileData(tileGreen, TILE_GREEN, 1, DMA);
    VDP_loadTileData(tileWinOn, TILE_WIN_ON, 1, DMA);
    VDP_loadTileData(tileWinOff, TILE_WIN_OFF, 1, DMA);
    VDP_loadTileData(tileCrack, TILE_CRACK, 1, DMA);
    VDP_loadTileData(tileCrackSmall, TILE_CRACK_SMALL, 1, DMA);
    VDP_loadTileData(tileCrackMid, TILE_CRACK_MID, 1, DMA);
    VDP_loadTileData(tileCrackLarge, TILE_CRACK_LARGE, 1, DMA);
    VDP_loadTileData(tileDust1, TILE_DUST_1, 1, DMA);
    VDP_loadTileData(tileDust2, TILE_DUST_2, 1, DMA);
    VDP_loadTileData(tileDust3, TILE_DUST_3, 1, DMA);
    VDP_loadTileData(tileRed, TILE_RED, 1, DMA);
    VDP_loadTileData(tileWhite, TILE_WHITE, 1, DMA);
    VDP_loadTileData(tileYellow, TILE_YELLOW, 1, DMA);
    VDP_loadTileData(tileARing, TILE_A_RING, 1, DMA);
    VDP_loadTileData(tileAUml, TILE_A_UML, 1, DMA);
    VDP_loadTileData(tileOUml, TILE_O_UML, 1, DMA);
}

static void playTone(u16 tone, u8 length)
{
    PSG_setTone(0, tone);
    PSG_setEnvelope(0, 2);
    PSG_setNoise(PSG_NOISE_TYPE_WHITE, PSG_NOISE_FREQ_CLOCK8);
    PSG_setEnvelope(3, length < 5 ? 9 : 6);
}

static void stopTone(void)
{
    PSG_setEnvelope(0, PSG_ENVELOPE_MIN);
    PSG_setEnvelope(1, PSG_ENVELOPE_MIN);
    PSG_setEnvelope(2, PSG_ENVELOPE_MIN);
    PSG_setEnvelope(3, PSG_ENVELOPE_MIN);
}

static void clearAll(void)
{
    VDP_clearPlane(BG_A, TRUE);
    VDP_clearPlane(BG_B, TRUE);
    hudFrameReady = FALSE;
}

static void drawPanel(u8 x, u8 y, u8 w, u8 h)
{
    VDP_fillTileMapRect(BG_A, attr(PAL0, TILE_DARK), x, y, w, h);
    VDP_fillTileMapRect(BG_A, attr(PAL0, TILE_WHITE), x, y, w, 1);
    VDP_fillTileMapRect(BG_A, attr(PAL0, TILE_WHITE), x, y + h - 1, w, 1);
    VDP_fillTileMapRect(BG_A, attr(PAL0, TILE_WHITE), x, y, 1, h);
    VDP_fillTileMapRect(BG_A, attr(PAL0, TILE_WHITE), x + w - 1, y, 1, h);
}

static void drawSkyline(void)
{
    const u8 heights[9] = {7, 11, 8, 13, 10, 12, 9, 14, 8};
    VDP_fillTileMapRect(BG_B, attr(PAL0, TILE_SKY), 0, 0, SCREEN_TILES_W, SCREEN_TILES_H);

    for (u8 i = 0; i < 9; i++)
    {
        const u8 x = i * 5;
        const u8 h = heights[i];
        VDP_fillTileMapRect(BG_B, attr(PAL0, TILE_DARK), x, FLOOR_Y - h, 5, h);
        if (x + 4 < SCREEN_TILES_W)
        {
            VDP_fillTileMapRect(BG_B, attr(PAL0, TILE_WIN_OFF), x + 4, FLOOR_Y - h + 1, 1, h - 1);
        }
        if ((i & 1) == 0)
        {
            VDP_setTileMapXY(BG_B, attr(PAL0, TILE_RED), x + 2, FLOOR_Y - h - 1);
        }
        else
        {
            VDP_setTileMapXY(BG_B, attr(PAL0, TILE_YELLOW), x + 1, FLOOR_Y - h);
            VDP_setTileMapXY(BG_B, attr(PAL0, TILE_YELLOW), x + 3, FLOOR_Y - h);
        }
    }

    for (u8 x = 0; x < SCREEN_TILES_W; x += 4)
    {
        VDP_setTileMapXY(BG_B, attr(PAL0, TILE_GREEN), x, FLOOR_Y - 1);
    }
}

static void drawRoad(void)
{
    VDP_fillTileMapRect(BG_B, attr(PAL0, TILE_ROAD), 0, FLOOR_Y + 1, SCREEN_TILES_W, 3);
    for (u8 x = 1; x < SCREEN_TILES_W; x += 6)
    {
        VDP_fillTileMapRect(BG_B, attr(PAL0, TILE_LINE), x, FLOOR_Y + 2, 3, 1);
    }
    VDP_fillTileMapRect(BG_B, attr(PAL0, TILE_WHITE), 0, FLOOR_Y, SCREEN_TILES_W, 1);
    VDP_fillTileMapRect(BG_B, attr(PAL0, TILE_DARK), 0, FLOOR_Y + 4, SCREEN_TILES_W, 1);
}

static void resetThreats(void)
{
    for (u8 i = 0; i < MAX_ENEMIES; i++)
    {
        enemies[i].active = FALSE;
        if (enemies[i].sprite != NULL) SPR_setVisibility(enemies[i].sprite, HIDDEN);
    }
    for (u8 i = 0; i < MAX_TANKS; i++)
    {
        tanks[i].active = FALSE;
        if (tanks[i].sprite != NULL) SPR_setVisibility(tanks[i].sprite, HIDDEN);
    }
    for (u8 i = 0; i < MAX_HELIS; i++)
    {
        helis[i].active = FALSE;
        if (helis[i].sprite != NULL) SPR_setVisibility(helis[i].sprite, HIDDEN);
    }
    for (u8 i = 0; i < MAX_SHOTS; i++)
    {
        shots[i].active = FALSE;
        if (shots[i].sprite != NULL) SPR_setVisibility(shots[i].sprite, HIDDEN);
    }
    for (u8 i = 0; i < MAX_PEOPLE; i++)
    {
        people[i].active = FALSE;
        if (people[i].sprite != NULL) SPR_setVisibility(people[i].sprite, HIDDEN);
    }
    for (u8 i = 0; i < MAX_EXPLOSIONS; i++)
    {
        explosions[i].active = FALSE;
        if (explosions[i].sprite != NULL) SPR_setVisibility(explosions[i].sprite, HIDDEN);
    }
}

static void initBuildings(void)
{
    const s16 xs[MAX_BUILDINGS] = {5, 12, 19, 27, 34};
    const u8 ws[MAX_BUILDINGS] = {6, 5, 6, 5, 4};
    const u8 hs[MAX_BUILDINGS] = {10, 13, 9, 11, 8};

    for (u8 i = 0; i < MAX_BUILDINGS; i++)
    {
        buildings[i].x = xs[i];
        buildings[i].w = ws[i];
        buildings[i].h = hs[i];
        buildings[i].y = FLOOR_Y - hs[i];
        buildings[i].damage = 0;
        for (u8 d = 0; d < MAX_DAMAGE_MARKS; d++)
        {
            buildings[i].damageX[d] = 0;
            buildings[i].damageY[d] = 0;
        }
        for (u8 p = 0; p < MAX_WINDOW_PEOPLE; p++)
        {
            const u8 col = 1 + ((p & 1) * 2);
            const u8 row = 1 + ((p / 2) * 2);
            buildings[i].personX[p] = buildings[i].x + (col < buildings[i].w ? col : 1);
            buildings[i].personY[p] = buildings[i].y + (row < buildings[i].h ? row : 1);
            buildings[i].personAlive[p] = (buildings[i].personY[p] < FLOOR_Y - 1);
            buildings[i].personEdible[p] = ((i + p) & 1) == 0;
        }
        buildings[i].colorPal = PAL0;
        buildings[i].crackTimer = 0;
        buildings[i].crackRows = 0;
        buildings[i].collapseTimer = 0;
        buildings[i].collapseRows = 0;
        buildings[i].cracking = FALSE;
        buildings[i].collapsing = FALSE;
        buildings[i].alive = TRUE;
    }
}

static void drawDamageTile(u8 x, u8 y, u8 tile)
{
    if (y >= HUD_TILES_H && y < FLOOR_Y && x < SCREEN_TILES_W)
    {
        VDP_setTileMapXY(BG_B, attr(PAL0, tile), x, y);
    }
}

static void drawCollapseDust(const Building *b, u8 y, u8 phase)
{
    if (y < HUD_TILES_H) y = HUD_TILES_H;
    if (y >= FLOOR_Y) y = FLOOR_Y - 1;

    for (u8 xx = 0; xx < b->w; xx++)
    {
        const u8 dust = TILE_DUST_1 + ((xx + phase) % 3);
        drawDamageTile(b->x + xx, y, dust);
        if ((xx + phase) & 1) drawDamageTile(b->x + xx, y + 1, TILE_DUST_1 + ((xx + phase + 1) % 3));
    }
}

static void drawBuildingCracks(const Building *b, u8 visibleH)
{
    const u8 waveRows = b->crackRows > (b->h - 2) ? b->h - 2 : b->crackRows;
    const u8 waveTop = (FLOOR_Y - 1 > waveRows) ? FLOOR_Y - 1 - waveRows : b->y;

    if (waveRows == 0) return;

    for (u8 yy = waveTop; yy < FLOOR_Y && yy < b->y + visibleH; yy++)
    {
        const u8 rel = yy - b->y;
        if (rel == 0 || rel >= visibleH) continue;

        const u8 xA = b->x + 1 + ((yy + b->crackRows) % (b->w - 1));
        const u8 xB = b->x + ((yy + b->crackRows + 2) % b->w);
        drawDamageTile(xA, yy, (rel & 1) ? TILE_CRACK_SMALL : TILE_CRACK_MID);
        if ((b->crackRows > 5) && ((yy + b->x) & 1)) drawDamageTile(xB, yy, TILE_CRACK_SMALL);
    }

}

static void drawBuildingChunks(const Building *b, u8 visibleH, u8 drawY)
{
    for (u8 d = 0; d < b->damage && d < MAX_DAMAGE_MARKS; d++)
    {
        const u8 cx = b->damageX[d];
        const u8 cy = b->damageY[d];
        if (cy < drawY || cy >= drawY + visibleH) continue;

        drawDamageTile(cx, cy, TILE_DARK);
        if (cx > b->x && (d & 1)) drawDamageTile(cx - 1, cy, TILE_WIN_OFF);
        if (cx + 1 < b->x + b->w && (d & 2)) drawDamageTile(cx + 1, cy, TILE_DARK);
        if (cy + 1 < drawY + visibleH) drawDamageTile(cx, cy + 1, (d & 1) ? TILE_DARK : TILE_WIN_OFF);
        if (cy > drawY && (d & 3) == 0) drawDamageTile(cx, cy - 1, TILE_WIN_OFF);
    }
}

static void drawBuilding(const Building *b)
{
    if (!b->alive && !b->collapsing) return;

    const u8 visibleH = b->collapseRows >= b->h ? 0 : b->h - b->collapseRows;
    const u8 drawY = b->collapsing ? b->y + b->collapseRows : b->y;
    if (visibleH == 0)
    {
        drawCollapseDust(b, FLOOR_Y - 1, b->collapseRows);
        return;
    }

    VDP_fillTileMapRect(BG_B, attr(PAL0, TILE_YELLOW), b->x, drawY, b->w, visibleH);
    VDP_fillTileMapRect(BG_B, attr(PAL0, TILE_WHITE), b->x, drawY, b->w, 1);
    VDP_fillTileMapRect(BG_B, attr(PAL0, TILE_DARK), b->x, drawY, 1, visibleH);
    if (!b->collapsing && visibleH > 2) VDP_fillTileMapRect(BG_B, attr(PAL0, TILE_RED), b->x + 1, drawY + visibleH - 2, b->w - 2, 1);
    if (b->w > 4 && visibleH > 2)
    {
        VDP_fillTileMapRect(BG_B, attr(PAL0, TILE_YELLOW), b->x + b->w - 1, drawY + 1, 1, visibleH - 2);
    }
    for (u8 yy = 1; yy < visibleH; yy += 2)
    {
        for (u8 xx = 1; xx + 1 < b->w; xx += 2)
        {
            const bool broken = ((xx + yy) & 5) == 0;
            VDP_setTileMapXY(BG_B, attr(PAL0, broken ? TILE_WIN_OFF : TILE_WIN_ON), b->x + xx, drawY + yy);
        }
    }
    if (!b->collapsing && visibleH > 5 && b->w > 3)
    {
        VDP_setTileMapXY(BG_B, attr(PAL0, TILE_DARK), b->x + 1, drawY + visibleH - 1);
        VDP_setTileMapXY(BG_B, attr(PAL0, TILE_WHITE), b->x + 2, drawY + visibleH - 1);
    }
    drawBuildingChunks(b, visibleH, drawY);
    if (b->cracking) drawBuildingCracks(b, visibleH);
    if (b->collapsing) drawCollapseDust(b, FLOOR_Y - 1, b->collapseRows);

    for (u8 p = 0; p < MAX_WINDOW_PEOPLE; p++)
    {
        if (!b->personAlive[p]) continue;
        if (b->personY[p] < drawY || b->personY[p] >= drawY + visibleH) continue;
        VDP_setTileMapXY(BG_B, attr(PAL0, b->personEdible[p] ? TILE_GREEN : TILE_WHITE), b->personX[p], b->personY[p]);
    }
}

static void drawBuildings(void)
{
    VDP_fillTileMapRect(BG_B, attr(PAL0, TILE_SKY), 0, HUD_TILES_H, SCREEN_TILES_W, FLOOR_Y - HUD_TILES_H);
    for (u8 i = 0; i < MAX_BUILDINGS; i++) drawBuilding(&buildings[i]);
    drawRoad();
}

static void drawHud(void)
{
    char buf[24];

    if (!hudFrameReady)
    {
        VDP_fillTileMapRect(BG_B, attr(PAL0, TILE_DARK), 0, 0, SCREEN_TILES_W, HUD_TILES_H);
        drawAsciiTextBg("P1", 1, 1);
        drawAsciiTextBg("SCORE", 5, 1);
        drawAsciiTextBg("LIV", 1, 2);
        hudScore = 0xFFFF;
        hudHealth = 0xFF;
        hudCity = 0xFF;
        hudFrameReady = TRUE;
    }

    if (hudScore != score)
    {
        VDP_fillTileMapRect(BG_B, attr(PAL0, TILE_DARK), 11, 1, 5, 1);
        intToStr(score, buf, 5);
        drawAsciiTextBg(buf, 11, 1);
        hudScore = score;
    }

    if (hudHealth != playerHealth)
    {
        for (u8 i = 0; i < PLAYER_MAX_HEALTH; i++)
        {
            VDP_setTileMapXY(BG_B, attr(PAL0, i < playerHealth ? TILE_RED : TILE_ROAD), 5 + i, 2);
        }
        hudHealth = playerHealth;
    }

    if (hudCity != currentCity)
    {
        VDP_fillTileMapRect(BG_B, attr(PAL0, TILE_DARK), 18, 2, SCREEN_TILES_W - 18, 1);
        drawTextSvBg(cities[currentCity], 18, 2);
        hudCity = currentCity;
    }
}

static void __attribute__((unused)) drawMonsterBody(s16 tx, s16 ty, u8 monster, bool big, bool punch)
{
    // Temporary tilemap monster: keep dimensions fixed so climb/punch never changes silhouette.
    const u8 pal = monsters[monster].pal;
    const u16 body = attr(pal, TILE_ROAD);
    const u16 shade = attr(pal, TILE_DARK);
    const u16 outline = attr(PAL0, TILE_DARK);
    const u16 eye = attr(PAL0, TILE_RED);
    const u8 h = 7;
    const u8 w = 6;

    if (tx < 0) tx = 0;
    if (ty < 0) ty = 0;
    if (tx > SCREEN_TILES_W - w - 2) tx = SCREEN_TILES_W - w - 2;
    if (ty > SCREEN_TILES_H - h) ty = SCREEN_TILES_H - h;

    VDP_fillTileMapRect(BG_A, outline, tx, ty, w, h);
    VDP_fillTileMapRect(BG_A, body, tx + 1, ty + 1, w - 2, h - 2);
    VDP_fillTileMapRect(BG_A, body, tx, ty + 2, 1, h - 4);
    VDP_fillTileMapRect(BG_A, body, tx + w - 1, ty + 2, 1, h - 4);
    VDP_fillTileMapRect(BG_A, shade, tx + 1, ty + 1, w - 2, 1);
    VDP_setTileMapXY(BG_A, eye, tx + 1, ty + 2);
    VDP_setTileMapXY(BG_A, eye, tx + w - 2, ty + 2);
    VDP_fillTileMapRect(BG_A, attr(PAL0, TILE_WHITE), tx + 2, ty + h - 2, w - 4, 1);

    if (monster == 1)
    {
        VDP_setTileMapXY(BG_A, body, tx, ty);
        VDP_setTileMapXY(BG_A, body, tx + w - 1, ty);
    }
    else if (monster == 2)
    {
        VDP_fillTileMapRect(BG_A, body, tx + w, ty + 2, 1, 3);
    }

    if (punch)
    {
        const s16 armX = tx + (player.dir > 0 ? w : -2);
        if (armX >= 0 && armX < SCREEN_TILES_W - 1)
        {
            VDP_fillTileMapRect(BG_A, body, armX, ty + 3, 2, 1);
            VDP_setTileMapXY(BG_A, attr(PAL0, TILE_WHITE), armX + (player.dir > 0 ? 1 : 0), ty + 3);
        }
    }
}

static void hidePlayerSprite(void)
{
    if (playerSprite != NULL) SPR_setVisibility(playerSprite, HIDDEN);
}

static void hideThreatSprites(void)
{
    for (u8 i = 0; i < MAX_ENEMIES; i++)
    {
        if (enemies[i].sprite != NULL) SPR_setVisibility(enemies[i].sprite, HIDDEN);
    }
    for (u8 i = 0; i < MAX_TANKS; i++)
    {
        if (tanks[i].sprite != NULL) SPR_setVisibility(tanks[i].sprite, HIDDEN);
    }
    for (u8 i = 0; i < MAX_HELIS; i++)
    {
        if (helis[i].sprite != NULL) SPR_setVisibility(helis[i].sprite, HIDDEN);
    }
    for (u8 i = 0; i < MAX_SHOTS; i++)
    {
        if (shots[i].sprite != NULL) SPR_setVisibility(shots[i].sprite, HIDDEN);
    }
    for (u8 i = 0; i < MAX_PEOPLE; i++)
    {
        if (people[i].sprite != NULL) SPR_setVisibility(people[i].sprite, HIDDEN);
    }
    for (u8 i = 0; i < MAX_EXPLOSIONS; i++)
    {
        if (explosions[i].sprite != NULL) SPR_setVisibility(explosions[i].sprite, HIDDEN);
    }
}

static void resetSpriteEngineState(void)
{
    SPR_reset();
    playerSprite = NULL;
    for (u8 i = 0; i < MAX_ENEMIES; i++) enemies[i].sprite = NULL;
    for (u8 i = 0; i < MAX_TANKS; i++) tanks[i].sprite = NULL;
    for (u8 i = 0; i < MAX_HELIS; i++) helis[i].sprite = NULL;
    for (u8 i = 0; i < MAX_SHOTS; i++) shots[i].sprite = NULL;
    for (u8 i = 0; i < MAX_PEOPLE; i++) people[i].sprite = NULL;
    for (u8 i = 0; i < MAX_EXPLOSIONS; i++) explosions[i].sprite = NULL;
}

static void showPlayerSprite(void)
{
    if (playerSprite == NULL)
    {
        playerSprite = SPR_addSprite(&monster_sprite, player.x, player.y, TILE_ATTR(PAL1, TRUE, FALSE, FALSE));
    }

    SPR_setVisibility(playerSprite, VISIBLE);
}

static void updatePlayerSprite(void)
{
    const u8 pal = monsters[player.monster].pal;
    u8 spriteFrame = player.monster * PLAYER_SPRITE_FRAMES;
    s16 spriteX = player.x;
    s16 spriteY = player.y + PLAYER_SPRITE_Y_OFFSET;
    const ClimbContact climbContact = getClimbContact();
    const bool visuallyClimbing = !player.grounded && climbContact.active;

    showPlayerSprite();
    SPR_setPalette(playerSprite, pal);
    if (player.stunTimer > 0) spriteFrame += POSE_STUNNED;
    else if (player.punching) spriteFrame += player.attackPose;
    else if (visuallyClimbing) spriteFrame += player.climbPose;
    else if (player.landTimer > 0) spriteFrame += (player.landTimer & 4) ? POSE_WALK_A : POSE_WALK_B;
    else if (player.walking) spriteFrame += (frame & 8) ? POSE_WALK_A : POSE_WALK_B;
    if (visuallyClimbing)
    {
        spriteX += climbContact.attackDir > 0 ? PLAYER_CLIMB_SPRITE_X_OFFSET : -PLAYER_CLIMB_SPRITE_X_OFFSET;
    }
    SPR_setFrame(playerSprite, spriteFrame);
    SPR_setHFlip(playerSprite, player.dir < 0);
    SPR_setPosition(playerSprite, spriteX, spriteY);
}

static void updateThreatSprites(void)
{
    for (u8 i = 0; i < MAX_ENEMIES; i++)
    {
        Enemy *e = &enemies[i];
        if (e->sprite == NULL)
        {
            e->sprite = SPR_addSprite(&enemy_sprite, e->x, e->y, TILE_ATTR(PAL0, TRUE, FALSE, FALSE));
        }
        SPR_setVisibility(e->sprite, e->active ? VISIBLE : HIDDEN);
        if (e->active)
        {
            SPR_setHFlip(e->sprite, e->speed < 0);
            SPR_setPosition(e->sprite, e->x, e->y + GROUND_SPRITE_Y_OFFSET);
        }
    }

    for (u8 i = 0; i < MAX_TANKS; i++)
    {
        Tank *t = &tanks[i];
        if (t->sprite == NULL)
        {
            t->sprite = SPR_addSprite(&tank_sprite, t->x, t->y, TILE_ATTR(PAL0, TRUE, FALSE, FALSE));
        }
        SPR_setVisibility(t->sprite, t->active ? VISIBLE : HIDDEN);
        if (t->active)
        {
            SPR_setHFlip(t->sprite, t->speed < 0);
            SPR_setPosition(t->sprite, t->x, t->y + GROUND_SPRITE_Y_OFFSET);
        }
    }

    for (u8 i = 0; i < MAX_HELIS; i++)
    {
        Helicopter *h = &helis[i];
        if (h->sprite == NULL)
        {
            h->sprite = SPR_addSprite(&helicopter_sprite, h->x, h->y, TILE_ATTR(PAL0, TRUE, FALSE, FALSE));
        }
        SPR_setVisibility(h->sprite, h->active ? VISIBLE : HIDDEN);
        if (h->active)
        {
            SPR_setHFlip(h->sprite, h->speed > 0);
            SPR_setPosition(h->sprite, h->x, h->y);
        }
    }

    for (u8 i = 0; i < MAX_SHOTS; i++)
    {
        Shot *s = &shots[i];
        if (s->sprite == NULL)
        {
            s->sprite = SPR_addSprite(&shot_sprite, s->x, s->y, TILE_ATTR(PAL0, TRUE, FALSE, FALSE));
        }
        SPR_setVisibility(s->sprite, s->active ? VISIBLE : HIDDEN);
        if (s->active) SPR_setPosition(s->sprite, s->x, s->y);
    }

    for (u8 i = 0; i < MAX_PEOPLE; i++)
    {
        Person *p = &people[i];
        if (p->sprite == NULL)
        {
            p->sprite = SPR_addSprite(&person_sprite, p->x, p->y, TILE_ATTR(PAL0, TRUE, FALSE, FALSE));
        }
        SPR_setVisibility(p->sprite, p->active ? VISIBLE : HIDDEN);
        if (p->active)
        {
            SPR_setHFlip(p->sprite, p->speed < 0);
            SPR_setPosition(p->sprite, p->x, p->y + (p->falling ? 0 : GROUND_SPRITE_Y_OFFSET));
        }
    }

    for (u8 i = 0; i < MAX_EXPLOSIONS; i++)
    {
        Explosion *ex = &explosions[i];
        if (ex->sprite == NULL)
        {
            ex->sprite = SPR_addSprite(&explosion_sprite, ex->x, ex->y, TILE_ATTR(PAL0, TRUE, FALSE, FALSE));
        }
        SPR_setVisibility(ex->sprite, ex->active ? VISIBLE : HIDDEN);
        if (ex->active)
        {
            SPR_setFrame(ex->sprite, ex->timer < 5 ? 1 : 0);
            SPR_setPosition(ex->sprite, ex->x, ex->y + (ex->y >= ((FLOOR_Y * 8) - 20) ? GROUND_SPRITE_Y_OFFSET : 0));
        }
    }
}

static void drawTitle(void)
{
    drawFullscreenImage(&title_screen);
}

static void drawSelectOverlay(void)
{
    static const u8 markerX[MAX_MONSTERS] = {5, 18, 31};
    const bool visible = (frame & 16) == 0;
    if (selectOverlayMonster == selectedMonster && selectOverlayVisible == visible)
    {
        return;
    }

    selectOverlayMonster = selectedMonster;
    selectOverlayVisible = visible;
    VDP_fillTileMapRect(BG_A, 0, 0, 22, SCREEN_TILES_W, 5);
    VDP_setTextPalette(PAL3);
    if (visible)
    {
        VDP_drawText("VALD", markerX[selectedMonster], 23);
        VDP_drawText(monsters[selectedMonster].name, markerX[selectedMonster] - 1, 24);
    }
}

static void drawSelect(void)
{
    drawFullscreenImage(&select_screen);
    PAL_setPalette(PAL3, paletteSelectOverlay, DMA);
    selectOverlayMonster = 0xFF;
    drawSelectOverlay();
}

static void drawIntro(void)
{
    hidePlayerSprite();
    hideThreatSprites();
    clearAll();
    drawSkyline();
    drawPanel(8, 9, 24, 5);
    VDP_setTextPalette(PAL0);
    drawTextSv(cities[currentCity], 15, 11);
}

static void startCity(void)
{
    initBuildings();
    resetThreats();
    player.x = 16;
    player.y = PLAYER_GROUND_Y;
    player.vy = 0;
    player.dir = 1;
    player.monster = selectedMonster;
    player.punching = FALSE;
    player.punchTimer = 0;
    player.attackPose = POSE_PUNCH;
    player.climbPose = POSE_CLIMB_UP;
    player.landTimer = 0;
    player.stunTimer = 0;
    player.gravityTimer = 0;
    player.walking = FALSE;
    player.grounded = TRUE;
    playerHealth = PLAYER_MAX_HEALTH;
    invulnerableTimer = 0;
    clearAll();
    drawSkyline();
    drawBuildings();
    VDP_fillTileMapRect(BG_A, 0, 0, 0, SCREEN_TILES_W, SCREEN_TILES_H);
    drawHud();
    updatePlayerSprite();
}

static void spawnEnemy(void)
{
    for (u8 i = 0; i < MAX_ENEMIES; i++)
    {
        if (!enemies[i].active)
        {
            const bool fromRight = ((frame >> 6) & 1) != 0;
            enemies[i].x = fromRight ? 320 : -24;
            enemies[i].y = (FLOOR_Y * 8) - 16;
            enemies[i].speed = fromRight ? -1 : 1;
            enemies[i].stepTimer = 0;
            enemies[i].active = TRUE;
            return;
        }
    }
}

static void spawnTank(void)
{
    for (u8 i = 0; i < MAX_TANKS; i++)
    {
        if (!tanks[i].active)
        {
            const bool fromRight = ((frame >> 7) & 1) != 0;
            tanks[i].x = fromRight ? 320 : -24;
            tanks[i].y = (FLOOR_Y * 8) - 16;
            tanks[i].speed = fromRight ? -1 : 1;
            tanks[i].stepTimer = 0;
            tanks[i].cooldown = 120;
            tanks[i].active = TRUE;
            return;
        }
    }
}

static void spawnHelicopter(void)
{
    for (u8 i = 0; i < MAX_HELIS; i++)
    {
        if (!helis[i].active)
        {
            const bool fromRight = ((frame >> 8) & 1) != 0;
            helis[i].x = fromRight ? 320 : -32;
            helis[i].y = 48 + ((frame >> 4) & 31);
            helis[i].speed = fromRight ? -1 : 1;
            helis[i].stepTimer = 0;
            helis[i].cooldown = 100;
            helis[i].burst = 0;
            helis[i].burstDelay = 0;
            helis[i].active = TRUE;
            return;
        }
    }
}

static void spawnShot(s16 x, s16 y, s16 dx, s16 dy)
{
    for (u8 i = 0; i < MAX_SHOTS; i++)
    {
        if (!shots[i].active)
        {
            shots[i].x = x;
            shots[i].y = y;
            shots[i].dx = dx;
            shots[i].dy = dy;
            shots[i].stepTimer = 0;
            shots[i].active = TRUE;
            playTone(280, 4);
            return;
        }
    }
}

static void spawnPerson(s16 x, s16 y, bool falling, bool edible)
{
    for (u8 i = 0; i < MAX_PEOPLE; i++)
    {
        if (!people[i].active)
        {
            people[i].x = x;
            people[i].y = y;
            people[i].vy = falling ? 1 : 0;
            people[i].speed = ((frame + i) & 1) ? 1 : -1;
            people[i].falling = falling;
            people[i].edible = edible;
            people[i].active = TRUE;
            return;
        }
    }
}

static void releaseWindowPeopleNear(Building *b, u8 hitX, u8 hitY)
{
    for (u8 p = 0; p < MAX_WINDOW_PEOPLE; p++)
    {
        if (!b->personAlive[p]) continue;

        const s16 dx = (s16)b->personX[p] - (s16)hitX;
        const s16 dy = (s16)b->personY[p] - (s16)hitY;
        if (dx < -1 || dx > 1 || dy < -1 || dy > 1) continue;

        b->personAlive[p] = FALSE;
        spawnPerson((s16)b->personX[p] * 8, (s16)b->personY[p] * 8, TRUE, b->personEdible[p]);
    }
}

static void releaseWindowPeopleAbove(Building *b, u8 tileY)
{
    for (u8 p = 0; p < MAX_WINDOW_PEOPLE; p++)
    {
        if (!b->personAlive[p]) continue;
        if (b->personY[p] >= tileY) continue;

        b->personAlive[p] = FALSE;
        spawnPerson((s16)b->personX[p] * 8, (s16)b->personY[p] * 8, TRUE, b->personEdible[p]);
    }
}

static void spawnExplosion(s16 x, s16 y)
{
    for (u8 i = 0; i < MAX_EXPLOSIONS; i++)
    {
        if (!explosions[i].active)
        {
            explosions[i].x = x + 4;
            explosions[i].y = y;
            explosions[i].timer = 10;
            explosions[i].active = TRUE;
            return;
        }
    }
}

static void hitPlayer(void)
{
    if (invulnerableTimer != 0) return;

    if (playerHealth > 0) playerHealth--;
    invulnerableTimer = 55;
    playTone(55, 12);

    if (playerHealth == 0)
    {
        state = STATE_GAME_OVER;
    }
}

static void incapacitatePlayer(void)
{
    hitPlayer();
    player.y = PLAYER_GROUND_Y;
    player.vy = 0;
    player.punching = FALSE;
    player.punchTimer = 0;
    player.walking = FALSE;
    player.grounded = TRUE;
    player.stunTimer = PLAYER_STUN_FRAMES;
    player.landTimer = PLAYER_STUN_FRAMES;
    playTone(48, 18);
}

static bool rectsOverlap(s16 ax, s16 ay, s16 aw, s16 ah, s16 bx, s16 by, s16 bw, s16 bh)
{
    return ax < bx + bw && ax + aw > bx && ay < by + bh && ay + ah > by;
}

static bool playerHitsRect(s16 x, s16 y, s16 w, s16 h)
{
    return rectsOverlap(player.x + PLAYER_HURT_X, player.y + PLAYER_HURT_Y, PLAYER_HURT_W, PLAYER_HURT_H, x, y, w, h);
}

static AttackBox getAttackBox(u16 joy)
{
    AttackBox box;
    const bool up = (joy & BUTTON_UP) != 0;
    const bool down = (joy & BUTTON_DOWN) != 0;
    s8 xDir = player.dir;

    if (joy & BUTTON_LEFT) xDir = -1;
    else if (joy & BUTTON_RIGHT) xDir = 1;

    box.xDir = xDir;
    box.yDir = up ? -1 : (down ? 1 : 0);
    box.pose = POSE_PUNCH;

    if (up && ((joy & (BUTTON_LEFT | BUTTON_RIGHT)) == 0))
    {
        box.x = player.x + 10;
        box.y = player.y - 10;
        box.w = 28;
        box.h = 26;
        box.pose = POSE_PUNCH_UP;
        return box;
    }

    box.x = player.x + (xDir > 0 ? 32 : -12);
    box.w = 28;
    box.h = 18;

    if (up)
    {
        box.y = player.y + 4;
        box.pose = POSE_PUNCH_DIAG_UP;
    }
    else if (down)
    {
        box.y = player.y + 34;
        box.pose = POSE_PUNCH_DIAG_DOWN;
    }
    else box.y = player.y + 20;

    return box;
}

static void updateThreats(void)
{
    const bool tailBouncing = (JOY_readJoypad(JOY_1) & BUTTON_A) && !player.grounded;

    if ((frame & 191) == 0) spawnEnemy();
    if ((frame & 511) == 128) spawnTank();
    if ((frame % 768) == 256) spawnHelicopter();

    for (u8 i = 0; i < MAX_ENEMIES; i++)
    {
        Enemy *e = &enemies[i];
        if (!e->active) continue;

        e->stepTimer++;
        if (e->stepTimer >= 2)
        {
            e->stepTimer = 0;
            e->x += e->speed;
        }
        if (e->x < -40 || e->x > 340)
        {
            e->active = FALSE;
            continue;
        }

        if (player.punching && rectsOverlap(player.attack.x, player.attack.y, player.attack.w, player.attack.h, e->x, e->y, 24, 16))
        {
            spawnExplosion(e->x, e->y);
            e->active = FALSE;
            score += 50;
            playTone(42, 12);
            continue;
        }

        if (playerHitsRect(e->x + ENEMY_HURT_X, e->y + ENEMY_HURT_Y, ENEMY_HURT_W, ENEMY_HURT_H))
        {
            if (tailBouncing)
            {
                spawnExplosion(e->x, e->y);
                score += 50;
                playTone(42, 12);
            }
            else
            {
                hitPlayer();
            }
            e->active = FALSE;
        }
    }

    for (u8 i = 0; i < MAX_TANKS; i++)
    {
        Tank *t = &tanks[i];
        if (!t->active) continue;

        t->stepTimer++;
        if (t->stepTimer >= 3)
        {
            t->stepTimer = 0;
            t->x += t->speed;
        }
        if (t->x < -40 || t->x > 340)
        {
            t->active = FALSE;
            continue;
        }

        if (t->cooldown > 0) t->cooldown--;
        if (t->cooldown == 0)
        {
            const s16 dx = (player.x < t->x) ? -1 : 1;
            spawnShot(t->x + 10, t->y - 2, dx, -1);
            t->cooldown = 150;
        }

        if (player.punching && rectsOverlap(player.attack.x, player.attack.y, player.attack.w, player.attack.h, t->x, t->y, 24, 16))
        {
            spawnExplosion(t->x, t->y);
            t->active = FALSE;
            score += 100;
            playTone(36, 14);
            continue;
        }

        if (playerHitsRect(t->x + ENEMY_HURT_X, t->y + ENEMY_HURT_Y, ENEMY_HURT_W + 4, ENEMY_HURT_H))
        {
            if (tailBouncing)
            {
                spawnExplosion(t->x, t->y);
                t->active = FALSE;
                score += 100;
                playTone(36, 14);
            }
            else
            {
                hitPlayer();
            }
        }
    }

    for (u8 i = 0; i < MAX_HELIS; i++)
    {
        Helicopter *h = &helis[i];
        if (!h->active) continue;

        h->stepTimer++;
        if (h->stepTimer >= 2)
        {
            h->stepTimer = 0;
            h->x += h->speed;
            if (h->x < 8)
            {
                h->x = 8;
                h->speed = 1;
            }
            else if (h->x > 280)
            {
                h->x = 280;
                h->speed = -1;
            }
        }

        if (h->burst > 0)
        {
            if (h->burstDelay > 0) h->burstDelay--;
            if (h->burstDelay == 0)
            {
                const s16 dx = (player.x < h->x) ? -1 : 1;
                const s16 muzzleX = h->x + (dx < 0 ? 4 : 24);
                spawnShot(muzzleX, h->y + 12, dx, 1);
                h->burst--;
                h->burstDelay = 5;
                if (h->burst == 0) h->cooldown = 150;
            }
        }
        else if (h->cooldown > 0)
        {
            h->cooldown--;
            if (h->cooldown == 0)
            {
                h->burst = 3;
                h->burstDelay = 1;
            }
        }

        if (player.punching && rectsOverlap(player.attack.x, player.attack.y, player.attack.w, player.attack.h, h->x, h->y, 32, 16))
        {
            spawnExplosion(h->x + 4, h->y);
            h->active = FALSE;
            score += 150;
            playTone(34, 14);
            continue;
        }

        if (playerHitsRect(h->x + 8, h->y + 4, 16, 8))
        {
            hitPlayer();
        }
    }

    for (u8 i = 0; i < MAX_SHOTS; i++)
    {
        Shot *s = &shots[i];
        if (!s->active) continue;

        s->stepTimer++;
        if (s->stepTimer >= 2)
        {
            s->stepTimer = 0;
            s->x += s->dx;
            s->y += s->dy;
        }
        if (s->x < -8 || s->x > 328 || s->y < 24 || s->y > 224)
        {
            s->active = FALSE;
            continue;
        }

        if (playerHitsRect(s->x + SHOT_HURT_X, s->y + SHOT_HURT_Y, SHOT_HURT_W, SHOT_HURT_H))
        {
            hitPlayer();
            s->active = FALSE;
        }
    }

    for (u8 i = 0; i < MAX_PEOPLE; i++)
    {
        Person *p = &people[i];
        if (!p->active) continue;

        if (p->falling)
        {
            p->y += p->vy;
            if (p->vy < 4) p->vy++;
            if (p->y >= (FLOOR_Y * 8) - 16)
            {
                p->y = (FLOOR_Y * 8) - 16;
                p->vy = 0;
                p->falling = FALSE;
            }
            continue;
        }

        if ((frame & 3) == 0) p->x += p->speed;
        if (p->x < 0)
        {
            p->x = 0;
            p->speed = 1;
        }
        if (p->x > 312)
        {
            p->x = 312;
            p->speed = -1;
        }
    }

    for (u8 i = 0; i < MAX_EXPLOSIONS; i++)
    {
        Explosion *ex = &explosions[i];
        if (!ex->active) continue;
        if (ex->timer > 0) ex->timer--;
        if (ex->timer == 0) ex->active = FALSE;
    }

    if (invulnerableTimer > 0) invulnerableTimer--;
}

static bool eatPerson(AttackBox attack)
{
    for (u8 i = 0; i < MAX_PEOPLE; i++)
    {
        Person *p = &people[i];
        if (!p->active) continue;

        const s16 eatX = p->x + PERSON_EAT_X;
        const s16 eatY = p->y + PERSON_EAT_Y;
        const s16 biteX = player.x + (player.dir > 0 ? PLAYER_BITE_X : PLAYER_W - PLAYER_BITE_X - PLAYER_BITE_W);
        const s16 biteY = player.y + PLAYER_BITE_Y;

        if (rectsOverlap(attack.x, attack.y, attack.w, attack.h, eatX, eatY, PERSON_EAT_W, PERSON_EAT_H) ||
            rectsOverlap(biteX, biteY, PLAYER_BITE_W, PLAYER_BITE_H, eatX, eatY, PERSON_EAT_W, PERSON_EAT_H))
        {
            p->active = FALSE;
            if (playerHealth < PLAYER_MAX_HEALTH) playerHealth++;
            score += 25;
            playTone(170, 8);
            return TRUE;
        }
    }

    return FALSE;
}

static bool attackEnemies(AttackBox attack)
{
    for (u8 i = 0; i < MAX_ENEMIES; i++)
    {
        Enemy *e = &enemies[i];
        if (!e->active) continue;

        if (rectsOverlap(attack.x, attack.y, attack.w, attack.h, e->x, e->y, 24, 16))
        {
            spawnExplosion(e->x, e->y);
            e->active = FALSE;
            score += 50;
            playTone(42, 12);
            return TRUE;
        }
    }

    for (u8 i = 0; i < MAX_TANKS; i++)
    {
        Tank *t = &tanks[i];
        if (!t->active) continue;

        if (rectsOverlap(attack.x, attack.y, attack.w, attack.h, t->x, t->y, 24, 16))
        {
            spawnExplosion(t->x, t->y);
            t->active = FALSE;
            score += 100;
            playTone(36, 14);
            return TRUE;
        }
    }

    for (u8 i = 0; i < MAX_HELIS; i++)
    {
        Helicopter *h = &helis[i];
        if (!h->active) continue;

        if (rectsOverlap(attack.x, attack.y, attack.w, attack.h, h->x, h->y, 32, 16))
        {
            spawnExplosion(h->x + 4, h->y);
            h->active = FALSE;
            score += 150;
            playTone(34, 14);
            return TRUE;
        }
    }

    for (u8 i = 0; i < MAX_SHOTS; i++)
    {
        Shot *s = &shots[i];
        if (!s->active) continue;

        if (rectsOverlap(attack.x, attack.y, attack.w, attack.h, s->x, s->y, 8, 8))
        {
            s->active = FALSE;
            score += 5;
            playTone(120, 5);
            return TRUE;
        }
    }

    return FALSE;
}

static void applyBuildingDamage(Building *b, u8 hitX, u8 hitY)
{
    const u8 mark = b->damage < MAX_DAMAGE_MARKS ? b->damage : MAX_DAMAGE_MARKS - 1;

    if (b->cracking || b->collapsing) return;

    if (b->damage < b->h + 2)
    {
        b->damageX[mark] = hitX;
        b->damageY[mark] = hitY;
        releaseWindowPeopleNear(b, hitX, hitY);
        b->damage++;
        score += 10;
        playTone(80 + (b->damage * 4), 9);
    }
    else
    {
        b->cracking = TRUE;
        b->crackTimer = BUILDING_CRACK_STEP_FRAMES;
        b->crackRows = 1;
        score += 100;
        playTone(54, 14);
    }
    drawBuildings();
    drawHud();
}

static bool damageBuildingAtAttack(AttackBox attack)
{
    for (u8 i = 0; i < MAX_BUILDINGS; i++)
    {
        Building *b = &buildings[i];
        const s16 bx = b->x * 8;
        const s16 by = b->y * 8;
        const s16 bw = b->w * 8;
        const s16 bh = b->h * 8;
        if (!b->alive) continue;
        if (b->cracking) continue;
        if (b->collapsing) continue;
        if (!rectsOverlap(attack.x, attack.y, attack.w, attack.h, bx, by, bw, bh)) continue;

        const s16 rawY = (attack.y + (attack.h / 2)) / 8;
        const u8 hitY = rawY < b->y ? b->y : (rawY >= FLOOR_Y ? FLOOR_Y - 1 : rawY);
        const u8 hitX = attack.xDir > 0 ? b->x : (b->x + b->w - 1);
        applyBuildingDamage(b, hitX, hitY);
        return TRUE;
    }

    return FALSE;
}

static void damageBuildings(ClimbContact contact, AttackBox attack)
{
    if (!contact.active) return;
    if (contact.building >= MAX_BUILDINGS) return;

    Building *b = &buildings[contact.building];
    if (!b->alive) return;
    if (b->cracking) return;
    if (b->collapsing) return;
    if ((player.y / 8) + 4 >= b->y)
    {
        const s16 rawY = (attack.y + (attack.h / 2)) / 8;
        const u8 hitY = rawY < b->y ? b->y : (rawY >= FLOOR_Y ? FLOOR_Y - 1 : rawY);
        const u8 hitX = contact.attackDir > 0 ? b->x : (b->x + b->w - 1);
        applyBuildingDamage(b, hitX, hitY);
    }
}

static bool cityCleared(void)
{
    for (u8 i = 0; i < MAX_BUILDINGS; i++)
    {
        if (buildings[i].alive || buildings[i].cracking || buildings[i].collapsing) return FALSE;
    }
    return TRUE;
}

static void updateBuildingCollapse(void)
{
    bool redraw = FALSE;

    for (u8 i = 0; i < MAX_BUILDINGS; i++)
    {
        Building *b = &buildings[i];
        if (b->cracking)
        {
            if (b->crackTimer > 0) b->crackTimer--;
            if (b->crackTimer != 0) continue;

            b->crackTimer = BUILDING_CRACK_STEP_FRAMES;
            if (b->crackRows < b->h)
            {
                b->crackRows++;
                playTone(62 + b->crackRows, 3);
            }
            else
            {
                const ClimbContact contact = getClimbContact();
                if (contact.active && contact.building == i && !player.grounded)
                {
                    incapacitatePlayer();
                }

                b->cracking = FALSE;
                b->collapsing = TRUE;
                b->collapseTimer = BUILDING_COLLAPSE_STEP_FRAMES;
                b->collapseRows = 1;
                releaseWindowPeopleAbove(b, b->y + 1);
                playTone(32, 14);
            }
            redraw = TRUE;
            continue;
        }

        if (!b->collapsing) continue;

        if (b->collapseTimer > 0) b->collapseTimer--;
        if (b->collapseTimer != 0) continue;

        b->collapseTimer = BUILDING_COLLAPSE_STEP_FRAMES;
        if (b->collapseRows < b->h)
        {
            b->collapseRows++;
            releaseWindowPeopleAbove(b, b->y + b->collapseRows);
            playTone(42 + (b->collapseRows * 2), 3);
        }
        else
        {
            b->collapsing = FALSE;
            b->alive = FALSE;
            b->collapseRows = b->h;
        }
        redraw = TRUE;
    }

    if (redraw) drawBuildings();
}

static RoofContact getRoofContact(void)
{
    RoofContact contact;
    const s16 center = player.x + (PLAYER_W / 2);
    const s16 feet = player.y + PLAYER_H;

    contact.active = FALSE;
    contact.y = PLAYER_GROUND_Y;
    contact.building = MAX_BUILDINGS;

    for (u8 i = 0; i < MAX_BUILDINGS; i++)
    {
        const Building *b = &buildings[i];
        const s16 bLeft = b->x * 8;
        const s16 bRight = (b->x + b->w) * 8;
        const s16 bTop = b->y * 8;
        if (!b->alive) continue;
        if (b->collapsing) continue;
        if (center < bLeft + 4 || center > bRight - 4) continue;
        if (feet < bTop - 6 || feet > bTop + 8) continue;

        contact.active = TRUE;
        contact.y = bTop - PLAYER_H;
        contact.building = i;
        return contact;
    }

    return contact;
}

static ClimbContact getClimbContact(void)
{
    ClimbContact contact;
    const s16 left = player.x;
    const s16 right = player.x + PLAYER_W;
    const s16 top = player.y;
    const s16 feet = player.y + PLAYER_H;
    contact.active = FALSE;
    contact.snapX = player.x;
    contact.attackDir = player.dir;
    contact.building = MAX_BUILDINGS;

    for (u8 i = 0; i < MAX_BUILDINGS; i++)
    {
        const Building *b = &buildings[i];
        const s16 bLeft = b->x * 8;
        const s16 bRight = (b->x + b->w) * 8;
        const s16 bTop = b->y * 8;
        const s16 bBottom = FLOOR_Y * 8;
        if (!b->alive) continue;
        if (b->collapsing) continue;

        if (feet < bTop - 6 || top > bBottom) continue;

        if (right >= bLeft - 6 && right <= bLeft + 10)
        {
            contact.active = TRUE;
            contact.snapX = bLeft - PLAYER_W + 4;
            contact.attackDir = 1;
            contact.building = i;
            return contact;
        }

        if (left <= bRight + 6 && left >= bRight - 10)
        {
            contact.active = TRUE;
            contact.snapX = bRight - 4;
            contact.attackDir = -1;
            contact.building = i;
            return contact;
        }
    }

    return contact;
}

static void updatePlayer(u16 joy, u16 pressed)
{
    ClimbContact climbContact = getClimbContact();
    RoofContact roofContact = getRoofContact();
    bool onFacade;
    bool climbing;
    bool onRoof;
    bool moved = FALSE;
    bool bounce = FALSE;
    const bool bouncing = (joy & BUTTON_A) && !player.grounded;

    if (player.stunTimer > 0)
    {
        player.stunTimer--;
        player.x += player.dir > 0 ? -1 : 1;
        player.y = PLAYER_GROUND_Y;
        player.vy = 0;
        player.punching = FALSE;
        player.walking = FALSE;
        player.grounded = TRUE;
        if (player.landTimer > 0) player.landTimer--;
        if (player.x < 0) player.x = 0;
        if (player.x > 280) player.x = 280;
        return;
    }

    if (joy & BUTTON_LEFT)
    {
        player.x -= 2;
        player.dir = -1;
        moved = TRUE;
    }
    if (joy & BUTTON_RIGHT)
    {
        player.x += 2;
        player.dir = 1;
        moved = TRUE;
    }

    climbContact = getClimbContact();
    roofContact = getRoofContact();
    onFacade = !bouncing && climbContact.active && player.y < PLAYER_GROUND_Y && !roofContact.active;
    climbing = !bouncing && (joy & BUTTON_UP) && climbContact.active;
    onRoof = roofContact.active && !onFacade && !climbing;

    if (onRoof)
    {
        player.y = roofContact.y;
        player.vy = 0;
        player.grounded = TRUE;
    }
    player.walking = moved && player.grounded;

    if (onFacade || climbing)
    {
        player.x = climbContact.snapX;
        player.dir = climbContact.attackDir;
        player.vy = 0;
        player.grounded = FALSE;
        player.walking = FALSE;
    }
    if (climbing && player.y > 40)
    {
        const Building *b = &buildings[climbContact.building];
        const s16 roofY = (b->y * 8) - PLAYER_H;

        player.y -= 2;
        player.climbPose = POSE_CLIMB_UP;
        if (player.y <= roofY)
        {
            const s16 bLeft = b->x * 8;
            const s16 bRight = (b->x + b->w) * 8;
            player.y = roofY;
            player.x = climbContact.attackDir > 0 ? bLeft - 20 : bRight - 28;
            player.vy = 0;
            player.grounded = TRUE;
            onFacade = FALSE;
            climbing = FALSE;
            onRoof = TRUE;
        }
    }
    else if ((joy & BUTTON_DOWN) && onFacade && player.y < PLAYER_GROUND_Y)
    {
        player.y += 2;
        player.climbPose = POSE_CLIMB_DOWN;
        if (player.y >= PLAYER_GROUND_Y)
        {
            player.y = PLAYER_GROUND_Y;
            player.grounded = TRUE;
        }
    }
    if ((pressed & BUTTON_A) && player.grounded)
    {
        player.vy = PLAYER_JUMP_VY;
        player.gravityTimer = 0;
        player.landTimer = 0;
        player.grounded = FALSE;
        onRoof = FALSE;
        playTone(210, 5);
    }
    if ((pressed & (BUTTON_B | BUTTON_C)) != 0)
    {
        const AttackBox attack = getAttackBox(joy);
        player.punching = TRUE;
        player.punchTimer = 10;
        player.attackPose = attack.pose;
        if (onFacade || climbing)
        {
            if (attack.xDir != climbContact.attackDir) player.attackPose = POSE_CLIMB_PUNCH_OUT;
            else if (attack.yDir < 0 && ((joy & (BUTTON_LEFT | BUTTON_RIGHT)) == 0)) player.attackPose = POSE_CLIMB_PUNCH_UP;
            else if (attack.yDir < 0) player.attackPose = POSE_CLIMB_PUNCH_DIAG_UP;
            else if (attack.yDir > 0) player.attackPose = POSE_CLIMB_PUNCH_DIAG_DOWN;
            else player.attackPose = POSE_CLIMB_PUNCH_IN;
        }
        player.attack = attack;
        if (!eatPerson(attack) && !attackEnemies(attack) && (onFacade || climbing))
        {
            if (attack.xDir == climbContact.attackDir) damageBuildings(climbContact, attack);
            else damageBuildingAtAttack(attack);
        }
    }

    if (!player.grounded && !onFacade && !climbing && !onRoof)
    {
        player.y += player.vy;
        player.gravityTimer++;
        const bool airBounce = (joy & BUTTON_A) != 0;
        const u8 gravityStep = airBounce ? PLAYER_BOUNCE_GRAVITY_STEP : PLAYER_GRAVITY_STEP;
        const s16 maxFall = airBounce ? PLAYER_BOUNCE_MAX_FALL : PLAYER_MAX_FALL;

        if (player.gravityTimer >= gravityStep)
        {
            player.gravityTimer = 0;
            if (player.vy < maxFall) player.vy++;
        }
        roofContact = getRoofContact();
        if (roofContact.active && player.vy >= 0)
        {
            player.y = roofContact.y;
            player.vy = 0;
            player.grounded = TRUE;
            player.landTimer = 12;
            playTone(180, 4);
            bounce = (joy & BUTTON_A) != 0;
        }
        else if (player.y >= PLAYER_GROUND_Y)
        {
            player.y = PLAYER_GROUND_Y;
            player.vy = 0;
            player.grounded = TRUE;
            player.landTimer = 12;
            playTone(180, 4);
            bounce = (joy & BUTTON_A) != 0;
        }
    }
    else if (!climbing && !onRoof && player.y < PLAYER_GROUND_Y && !getClimbContact().active)
    {
        player.grounded = FALSE;
        player.vy = 2;
    }

    if (player.x < 0) player.x = 0;
    if (player.x > 280) player.x = 280;

    if (player.punchTimer > 0)
    {
        player.punchTimer--;
        if (player.punchTimer == 0) player.punching = FALSE;
    }
    if (player.landTimer > 0) player.landTimer--;

    if (bounce && player.grounded)
    {
        player.vy = PLAYER_BOUNCE_VY;
        player.gravityTimer = 0;
        player.grounded = FALSE;
        player.landTimer = 0;
        playTone(220, 4);
    }
}

static void redrawPlayFrame(void)
{
    VDP_fillTileMapRect(BG_A, 0, 0, 0, SCREEN_TILES_W, SCREEN_TILES_H);
    drawHud();

    updatePlayerSprite();
    updateThreatSprites();
    SPR_setVisibility(playerSprite, ((invulnerableTimer & 4) == 0) ? VISIBLE : HIDDEN);
}

static void drawClear(void)
{
    hidePlayerSprite();
    hideThreatSprites();
    clearAll();
    drawSkyline();
    drawPanel(6, 8, 28, 8);
    VDP_setTextPalette(PAL0);
    drawTextSv("STADEN ÄR RIVEN", 12, 10);
    drawTextSv("START FÖR NÄSTA", 12, 12);
}

static void drawGameOver(void)
{
    hidePlayerSprite();
    hideThreatSprites();
    clearAll();
    drawSkyline();
    drawPanel(8, 8, 24, 8);
    VDP_setTextPalette(PAL0);
    VDP_drawText("GAME OVER", 15, 10);
    drawTextSv("POLISEN TOG DIG", 12, 12);
    VDP_drawText("START", 17, 14);
}

static void handleState(u16 joy)
{
    const u16 pressed = joy & ~previousJoy;

    switch (state)
    {
        case STATE_TITLE:
            if (pressed & BUTTON_START)
            {
                state = STATE_SELECT;
                drawSelect();
                playTone(120, 8);
            }
            break;

        case STATE_SELECT:
            drawSelectOverlay();
            if (pressed & BUTTON_LEFT)
            {
                selectedMonster = (selectedMonster + MAX_MONSTERS - 1) % MAX_MONSTERS;
                drawSelectOverlay();
                playTone(180, 4);
            }
            if (pressed & BUTTON_RIGHT)
            {
                selectedMonster = (selectedMonster + 1) % MAX_MONSTERS;
                drawSelectOverlay();
                playTone(160, 4);
            }
            if (pressed & BUTTON_START)
            {
                state = STATE_INTRO;
                loadGameVideo();
                drawIntro();
                playTone(100, 10);
            }
            break;

        case STATE_INTRO:
            if ((frame & 63) == 0 || (pressed & BUTTON_START))
            {
                state = STATE_PLAY;
                startCity();
            }
            break;

        case STATE_PLAY:
            updatePlayer(joy, pressed);
            updateThreats();
            updateBuildingCollapse();
            if (state == STATE_GAME_OVER)
            {
                resetSpriteEngineState();
                drawGameOver();
                break;
            }
            redrawPlayFrame();
            if (cityCleared())
            {
                state = STATE_CLEAR;
                drawClear();
            }
            break;

        case STATE_CLEAR:
            if (pressed & BUTTON_START)
            {
                currentCity = (currentCity + 1) % (sizeof(cities) / sizeof(cities[0]));
                state = STATE_INTRO;
                drawIntro();
            }
            break;

        case STATE_GAME_OVER:
            if (pressed & BUTTON_START)
            {
                score = 0;
                currentCity = 0;
                state = STATE_TITLE;
                drawTitle();
            }
            break;
    }

    previousJoy = joy;
}

int main(bool hardReset)
{
    SYS_disableInts();
    VDP_setScreenWidth320();
    VDP_setPlaneSize(64, 32, TRUE);
    loadGameVideo();
    SPR_init();
    PSG_reset();
    SYS_enableInts();

    drawTitle();

    while (TRUE)
    {
        const u16 joy = JOY_readJoypad(JOY_1);
        handleState(joy);

        if ((frame & 7) == 0) stopTone();
        frame++;
        SPR_update();
        SYS_doVBlankProcess();
    }

    return 0;
}
