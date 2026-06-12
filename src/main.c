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
#define MAX_SHOTS 4
#define MAX_PEOPLE 4
#define MAX_DAMAGE_MARKS 16
#define PLAYER_MAX_HEALTH 8
#define PLAYER_W 48
#define PLAYER_H 56
#define PLAYER_GROUND_Y ((FLOOR_Y - 7) * 8)

#define TILE_SKY     (TILE_USER_INDEX + 0)
#define TILE_DARK    (TILE_USER_INDEX + 1)
#define TILE_ROAD    (TILE_USER_INDEX + 2)
#define TILE_LINE    (TILE_USER_INDEX + 3)
#define TILE_GREEN   (TILE_USER_INDEX + 4)
#define TILE_WIN_ON  (TILE_USER_INDEX + 5)
#define TILE_WIN_OFF (TILE_USER_INDEX + 6)
#define TILE_CRACK   (TILE_USER_INDEX + 7)
#define TILE_RED     (TILE_USER_INDEX + 8)
#define TILE_WHITE   (TILE_USER_INDEX + 9)
#define TILE_YELLOW  (TILE_USER_INDEX + 10)
#define TILE_A_RING  (TILE_USER_INDEX + 11)
#define TILE_A_UML   (TILE_USER_INDEX + 12)
#define TILE_O_UML   (TILE_USER_INDEX + 13)

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
    s16 vy;
    s16 dir;
    u8 monster;
    u8 punching;
    u8 punchTimer;
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
    u8 colorPal;
    bool alive;
} Building;

typedef struct
{
    s16 x;
    s16 y;
    s16 speed;
    u8 stepTimer;
    u8 cooldown;
    bool active;
    Sprite *sprite;
} Enemy;

typedef struct
{
    s16 x;
    s16 y;
    s16 speed;
    u8 stepTimer;
    bool active;
    Sprite *sprite;
} Shot;

typedef struct
{
    s16 x;
    s16 y;
    s16 w;
    s16 h;
    s8 xDir;
    s8 yDir;
} AttackBox;

typedef struct
{
    s16 x;
    s16 y;
    s16 speed;
    bool active;
    Sprite *sprite;
} Person;

typedef struct
{
    bool active;
    s16 snapX;
    s8 attackDir;
    u8 building;
} ClimbContact;

static const u32 tileSky[8]     = {0x11111111,0x11111111,0x11111111,0x11111111,0x11111111,0x11111111,0x11111111,0x11111111};
static const u32 tileDark[8]    = {0x22222222,0x22222222,0x22222222,0x22222222,0x22222222,0x22222222,0x22222222,0x22222222};
static const u32 tileRoad[8]    = {0x33333333,0x33333333,0x33333333,0x33333333,0x33333333,0x33333333,0x33333333,0x33333333};
static const u32 tileLine[8]    = {0x33333333,0x33333333,0x33333333,0x33333333,0x99999999,0x99999999,0x33333333,0x33333333};
static const u32 tileGreen[8]   = {0x00440000,0x04444000,0x44444400,0x04444000,0x00440000,0x00A00000,0x00A00000,0x00A00000};
static const u32 tileWinOn[8]   = {0x00000000,0x06666000,0x06666000,0x00000000,0x07777000,0x07777000,0x00000000,0x00000000};
static const u32 tileWinOff[8]  = {0x00000000,0x02222000,0x02222000,0x00000000,0x02222000,0x02222000,0x00000000,0x00000000};
static const u32 tileCrack[8]   = {0x00080000,0x00088000,0x00880000,0x08800000,0x00880000,0x00088000,0x00080000,0x00000000};
static const u32 tileRed[8]     = {0x88888888,0x88888888,0x88888888,0x88888888,0x88888888,0x88888888,0x88888888,0x88888888};
static const u32 tileWhite[8]   = {0x99999999,0x99999999,0x99999999,0x99999999,0x99999999,0x99999999,0x99999999,0x99999999};
static const u32 tileYellow[8]  = {0x77777777,0x77777777,0x77777777,0x77777777,0x77777777,0x77777777,0x77777777,0x77777777};
static const u32 tileARing[8]   = {0x000FF000,0x00F00F00,0x000FF000,0x00F00F00,0x0F0000F0,0x0FFFFFF0,0x0F0000F0,0x0F0000F0};
static const u32 tileAUml[8]    = {0x00F00F00,0x00000000,0x00F00F00,0x0F0000F0,0x0FFFFFF0,0x0F0000F0,0x0F0000F0,0x00000000};
static const u32 tileOUml[8]    = {0x00F00F00,0x00000000,0x00FFFF00,0x0F0000F0,0x0F0000F0,0x0F0000F0,0x00FFFF00,0x00000000};

static const u16 palette0[16] =
{
    RGB3_3_3_TO_VDPCOLOR(0,0,0),
    RGB3_3_3_TO_VDPCOLOR(0,1,7),
    RGB3_3_3_TO_VDPCOLOR(0,0,0),
    RGB3_3_3_TO_VDPCOLOR(3,3,3),
    RGB3_3_3_TO_VDPCOLOR(0,6,1),
    RGB3_3_3_TO_VDPCOLOR(0,4,1),
    RGB3_3_3_TO_VDPCOLOR(4,5,7),
    RGB3_3_3_TO_VDPCOLOR(7,6,0),
    RGB3_3_3_TO_VDPCOLOR(7,1,0),
    RGB3_3_3_TO_VDPCOLOR(7,7,7),
    RGB3_3_3_TO_VDPCOLOR(3,2,0),
    RGB3_3_3_TO_VDPCOLOR(1,1,2),
    RGB3_3_3_TO_VDPCOLOR(0,0,4),
    RGB3_3_3_TO_VDPCOLOR(7,7,0),
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
static Shot shots[MAX_SHOTS];
static Person people[MAX_PEOPLE];
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

static ClimbContact getClimbContact(void);

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
}

static void resetThreats(void)
{
    for (u8 i = 0; i < MAX_ENEMIES; i++)
    {
        enemies[i].active = FALSE;
        if (enemies[i].sprite != NULL) SPR_setVisibility(enemies[i].sprite, HIDDEN);
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
        buildings[i].colorPal = PAL0;
        buildings[i].alive = TRUE;
    }
}

static void drawBuilding(const Building *b)
{
    if (!b->alive) return;

    VDP_fillTileMapRect(BG_B, attr(PAL0, TILE_YELLOW), b->x, b->y, b->w, b->h);
    for (u8 yy = 1; yy < b->h; yy += 2)
    {
        for (u8 xx = 1; xx + 1 < b->w; xx += 2)
        {
            const bool broken = ((xx + yy + b->damage) & 3) == 0;
            VDP_setTileMapXY(BG_B, attr(PAL0, broken ? TILE_WIN_OFF : TILE_WIN_ON), b->x + xx, b->y + yy);
        }
    }
    for (u8 d = 0; d < b->damage && d < MAX_DAMAGE_MARKS; d++)
    {
        const u8 cx = b->damageX[d];
        const u8 cy = b->damageY[d];
        if (cy < FLOOR_Y) VDP_setTileMapXY(BG_B, attr(PAL0, TILE_CRACK), cx, cy);
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

static void drawMonsterBody(s16 tx, s16 ty, u8 monster, bool big, bool punch)
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
    for (u8 i = 0; i < MAX_SHOTS; i++)
    {
        if (shots[i].sprite != NULL) SPR_setVisibility(shots[i].sprite, HIDDEN);
    }
    for (u8 i = 0; i < MAX_PEOPLE; i++)
    {
        if (people[i].sprite != NULL) SPR_setVisibility(people[i].sprite, HIDDEN);
    }
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
    u8 frame = player.monster * 3;

    showPlayerSprite();
    SPR_setPalette(playerSprite, pal);
    if (player.punching) frame += 2;
    else if (!player.grounded && getClimbContact().active) frame += 1;
    SPR_setFrame(playerSprite, frame);
    SPR_setHFlip(playerSprite, player.dir < 0);
    SPR_setPosition(playerSprite, player.x, player.y);
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
            SPR_setPosition(e->sprite, e->x, e->y);
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
            SPR_setPosition(p->sprite, p->x, p->y);
        }
    }
}

static void drawTitle(void)
{
    hidePlayerSprite();
    hideThreatSprites();
    clearAll();
    VDP_fillTileMapRect(BG_B, attr(PAL0, TILE_SKY), 0, 0, SCREEN_TILES_W, SCREEN_TILES_H);
    drawSkyline();
    VDP_fillTileMapRect(BG_A, attr(PAL0, TILE_SKY), 0, 0, SCREEN_TILES_W, SCREEN_TILES_H);

    VDP_setTextPalette(PAL0);
    VDP_drawText("VANDALER", 12, 6);
    drawTextSv("SVENSK STADSFÖRSTÖRELSE", 7, 10);
    VDP_drawText("START", 17, 18);
    drawMonsterBody(4, 11, 0, TRUE, TRUE);
    drawMonsterBody(29, 11, 2, TRUE, FALSE);
}

static void drawSelect(void)
{
    hidePlayerSprite();
    hideThreatSprites();
    clearAll();
    VDP_fillTileMapRect(BG_B, attr(PAL0, TILE_DARK), 0, 0, SCREEN_TILES_W, SCREEN_TILES_H);
    VDP_setTextPalette(PAL0);
    drawTextSv("VÄLJ MONSTER", 14, 3);

    for (u8 i = 0; i < MAX_MONSTERS; i++)
    {
        const s16 x = 5 + (i * 11);
        VDP_drawText(monsters[i].name, x, 7);
        VDP_drawText(monsters[i].species, x + 1, 8);
        drawMonsterBody(x, 11, i, TRUE, selectedMonster == i);
        if (selectedMonster == i)
        {
            VDP_drawText(">", x - 2, 13);
            VDP_drawText("<", x + 6, 13);
        }
    }
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
            enemies[i].cooldown = 90 + (i * 20);
            enemies[i].active = TRUE;
            return;
        }
    }
}

static void spawnShot(s16 x, s16 y, s16 speed)
{
    for (u8 i = 0; i < MAX_SHOTS; i++)
    {
        if (!shots[i].active)
        {
            shots[i].x = x;
            shots[i].y = y;
            shots[i].speed = speed;
            shots[i].stepTimer = 0;
            shots[i].active = TRUE;
            playTone(280, 4);
            return;
        }
    }
}

static void spawnPersonNearBuilding(const Building *b)
{
    for (u8 i = 0; i < MAX_PEOPLE; i++)
    {
        if (!people[i].active)
        {
            people[i].x = (b->x * 8) + 8;
            people[i].y = (FLOOR_Y * 8) - 16;
            people[i].speed = ((frame + i) & 1) ? 1 : -1;
            people[i].active = TRUE;
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

static bool rectsOverlap(s16 ax, s16 ay, s16 aw, s16 ah, s16 bx, s16 by, s16 bw, s16 bh)
{
    return ax < bx + bw && ax + aw > bx && ay < by + bh && ay + ah > by;
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

    if (up && ((joy & (BUTTON_LEFT | BUTTON_RIGHT)) == 0))
    {
        box.x = player.x + 10;
        box.y = player.y - 10;
        box.w = 28;
        box.h = 26;
        return box;
    }

    box.x = player.x + (xDir > 0 ? 32 : -12);
    box.w = 28;
    box.h = 18;

    if (up) box.y = player.y + 4;
    else if (down) box.y = player.y + 34;
    else box.y = player.y + 20;

    return box;
}

static void updateThreats(void)
{
    if ((frame & 191) == 0) spawnEnemy();

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

        if (e->cooldown > 0) e->cooldown--;
        if (e->cooldown == 0)
        {
            const s16 shotSpeed = (player.x < e->x) ? -1 : 1;
            spawnShot(e->x + 8, e->y - 18, shotSpeed);
            e->cooldown = 130;
        }

        if (rectsOverlap(player.x, player.y, 40, 48, e->x, e->y, 24, 16))
        {
            hitPlayer();
            e->active = FALSE;
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
            s->x += s->speed;
        }
        if (s->x < -8 || s->x > 328)
        {
            s->active = FALSE;
            continue;
        }

        if (rectsOverlap(player.x, player.y, 40, 48, s->x, s->y, 8, 8))
        {
            hitPlayer();
            s->active = FALSE;
        }
    }

    for (u8 i = 0; i < MAX_PEOPLE; i++)
    {
        Person *p = &people[i];
        if (!p->active) continue;

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

    if (invulnerableTimer > 0) invulnerableTimer--;
}

static bool eatPerson(AttackBox attack)
{
    for (u8 i = 0; i < MAX_PEOPLE; i++)
    {
        Person *p = &people[i];
        if (!p->active) continue;

        if (rectsOverlap(attack.x, attack.y, attack.w, attack.h, p->x, p->y, 8, 16))
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
            e->active = FALSE;
            score += 50;
            playTone(42, 12);
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

static void damageBuildings(ClimbContact contact, AttackBox attack)
{
    if (!contact.active) return;
    if (contact.building >= MAX_BUILDINGS) return;

    Building *b = &buildings[contact.building];
    if (!b->alive) return;
    if ((player.y / 8) + 4 >= b->y)
    {
        const u8 mark = b->damage < MAX_DAMAGE_MARKS ? b->damage : MAX_DAMAGE_MARKS - 1;
        const s16 rawY = (attack.y + (attack.h / 2)) / 8;
        const u8 hitY = rawY < b->y ? b->y : (rawY >= FLOOR_Y ? FLOOR_Y - 1 : rawY);
        const u8 hitX = contact.attackDir > 0 ? b->x : (b->x + b->w - 1);

        if (b->damage < b->h - 2)
        {
            b->damageX[mark] = hitX;
            b->damageY[mark] = hitY;
            b->damage++;
            score += 10;
            playTone(80 + (b->damage * 4), 9);
            if ((b->damage & 1) != 0) spawnPersonNearBuilding(b);
        }
        else
        {
            b->alive = FALSE;
            score += 100;
            playTone(32, 14);
        }
        drawBuildings();
        drawHud();
    }
}

static bool cityCleared(void)
{
    for (u8 i = 0; i < MAX_BUILDINGS; i++)
    {
        if (buildings[i].alive) return FALSE;
    }
    return TRUE;
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

        if (feet < bTop + 12 || top > bBottom) continue;

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
    const ClimbContact climbContact = getClimbContact();
    const bool onFacade = climbContact.active && player.y < PLAYER_GROUND_Y;
    const bool climbing = (joy & BUTTON_UP) && climbContact.active;

    if (joy & BUTTON_LEFT)
    {
        player.x -= 2;
        player.dir = -1;
    }
    if (joy & BUTTON_RIGHT)
    {
        player.x += 2;
        player.dir = 1;
    }
    if (onFacade || climbing)
    {
        player.x = climbContact.snapX;
        player.dir = climbContact.attackDir;
        player.vy = 0;
        player.grounded = FALSE;
    }
    if (climbing && player.y > 40)
    {
        player.y -= 2;
    }
    else if ((joy & BUTTON_DOWN) && onFacade && player.y < PLAYER_GROUND_Y)
    {
        player.y += 2;
        if (player.y >= PLAYER_GROUND_Y)
        {
            player.y = PLAYER_GROUND_Y;
            player.grounded = TRUE;
        }
    }
    if ((pressed & BUTTON_A) && player.grounded)
    {
        player.vy = -8;
        player.grounded = FALSE;
        playTone(210, 5);
    }
    if ((pressed & (BUTTON_B | BUTTON_C)) != 0)
    {
        const AttackBox attack = getAttackBox(joy);
        player.punching = TRUE;
        player.punchTimer = 10;
        if (!eatPerson(attack) && !attackEnemies(attack) && (onFacade || climbing)) damageBuildings(climbContact, attack);
    }

    if (!player.grounded && !onFacade && !climbing)
    {
        player.y += player.vy;
        player.vy++;
        if (player.y >= PLAYER_GROUND_Y)
        {
            player.y = PLAYER_GROUND_Y;
            player.vy = 0;
            player.grounded = TRUE;
        }
    }
    else if (!climbing && player.y < PLAYER_GROUND_Y && !getClimbContact().active)
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
            if (pressed & BUTTON_LEFT)
            {
                selectedMonster = (selectedMonster + MAX_MONSTERS - 1) % MAX_MONSTERS;
                drawSelect();
                playTone(180, 4);
            }
            if (pressed & BUTTON_RIGHT)
            {
                selectedMonster = (selectedMonster + 1) % MAX_MONSTERS;
                drawSelect();
                playTone(160, 4);
            }
            if (pressed & BUTTON_START)
            {
                state = STATE_INTRO;
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
            if (state == STATE_GAME_OVER)
            {
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
    PAL_setPalette(PAL0, palette0, DMA);
    PAL_setPalette(PAL1, paletteApe, DMA);
    PAL_setPalette(PAL2, paletteWolf, DMA);
    PAL_setPalette(PAL3, paletteLizard, DMA);
    loadTiles();
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
