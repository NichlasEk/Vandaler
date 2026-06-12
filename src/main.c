#include <genesis.h>
#include <psg.h>

#define SCREEN_TILES_W 40
#define SCREEN_TILES_H 28
#define FLOOR_Y 23
#define MAX_BUILDINGS 5
#define MAX_MONSTERS 3

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
    STATE_CLEAR
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
    u8 colorPal;
    bool alive;
} Building;

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
    RGB3_3_3_TO_VDPCOLOR(0,0,0),
    RGB3_3_3_TO_VDPCOLOR(0,0,0),
    RGB3_3_3_TO_VDPCOLOR(0,0,0),
    RGB3_3_3_TO_VDPCOLOR(0,0,0),
    RGB3_3_3_TO_VDPCOLOR(0,0,0)
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

static const char *cities[] = {"ÖREBRO", "STOCKHOLM", "GÖTEBORG", "MALMÖ", "KIRUNA"};

static GameState state = STATE_TITLE;
static Player player;
static Building buildings[MAX_BUILDINGS];
static u8 selectedMonster = 0;
static u8 currentCity = 0;
static u16 frame = 0;
static u16 score = 0;
static u16 previousJoy = 0;

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
    VDP_fillTileMapRect(BG_A, attr(PAL0, TILE_ROAD), 0, FLOOR_Y + 1, SCREEN_TILES_W, 3);
    for (u8 x = 1; x < SCREEN_TILES_W; x += 6)
    {
        VDP_fillTileMapRect(BG_A, attr(PAL0, TILE_LINE), x, FLOOR_Y + 2, 3, 1);
    }
    VDP_fillTileMapRect(BG_A, attr(PAL0, TILE_WHITE), 0, FLOOR_Y, SCREEN_TILES_W, 1);
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
        buildings[i].colorPal = PAL0;
        buildings[i].alive = TRUE;
    }
}

static void drawBuilding(const Building *b)
{
    if (!b->alive) return;

    VDP_fillTileMapRect(BG_A, attr(PAL0, TILE_YELLOW), b->x, b->y, b->w, b->h);
    for (u8 yy = 1; yy < b->h; yy += 2)
    {
        for (u8 xx = 1; xx + 1 < b->w; xx += 2)
        {
            const bool broken = ((xx + yy + b->damage) & 3) == 0;
            VDP_setTileMapXY(BG_A, attr(PAL0, broken ? TILE_WIN_OFF : TILE_WIN_ON), b->x + xx, b->y + yy);
        }
    }
    for (u8 d = 0; d < b->damage; d++)
    {
        const u8 cx = b->x + 1 + ((d * 2) % (b->w - 1));
        const u8 cy = b->y + 1 + d;
        if (cy < FLOOR_Y) VDP_setTileMapXY(BG_A, attr(PAL0, TILE_CRACK), cx, cy);
    }
}

static void drawBuildings(void)
{
    VDP_fillTileMapRect(BG_A, attr(PAL0, TILE_SKY), 0, 0, SCREEN_TILES_W, FLOOR_Y);
    for (u8 i = 0; i < MAX_BUILDINGS; i++) drawBuilding(&buildings[i]);
    drawRoad();
}

static void drawHud(void)
{
    char buf[24];
    VDP_fillTileMapRect(BG_A, attr(PAL0, TILE_SKY), 0, 0, SCREEN_TILES_W, 2);
    VDP_setTextPalette(PAL0);
    VDP_drawText("1P", 1, 0);
    intToStr(score, buf, 1);
    VDP_drawText(buf, 7, 0);
    drawTextSv(cities[currentCity], 22, 0);
}

static void drawMonsterBody(s16 tx, s16 ty, u8 monster, bool big, bool punch)
{
    const u8 pal = monsters[monster].pal;
    const u16 body = attr(pal, TILE_YELLOW);
    const u16 dark = attr(pal, TILE_DARK);
    const u16 eye = attr(pal, TILE_RED);
    const u8 h = big ? 6 : 4;
    const u8 w = big ? 5 : 4;

    VDP_fillTileMapRect(BG_A, body, tx + 1, ty + 1, w - 2, h - 1);
    VDP_fillTileMapRect(BG_A, body, tx, ty + 2, 1, h - 2);
    VDP_fillTileMapRect(BG_A, body, tx + w - 1, ty + 2, 1, h - 2);
    VDP_fillTileMapRect(BG_A, dark, tx + 1, ty, w - 2, 1);
    VDP_setTileMapXY(BG_A, eye, tx + 1, ty + 1);
    VDP_setTileMapXY(BG_A, eye, tx + w - 2, ty + 1);
    VDP_fillTileMapRect(BG_A, attr(pal, TILE_WHITE), tx + 1, ty + h - 1, w - 2, 1);

    if (monster == 1)
    {
        VDP_setTileMapXY(BG_A, body, tx, ty);
        VDP_setTileMapXY(BG_A, body, tx + w - 1, ty);
    }
    else if (monster == 2)
    {
        VDP_fillTileMapRect(BG_A, attr(pal, TILE_GREEN), tx + w, ty + 2, 1, 3);
    }

    if (punch)
    {
        VDP_fillTileMapRect(BG_A, attr(pal, TILE_WHITE), tx + w, ty + 3, 2, 1);
    }
}

static void drawTitle(void)
{
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
    clearAll();
    drawSkyline();
    VDP_fillTileMapRect(BG_A, attr(PAL0, TILE_DARK), 8, 10, 24, 3);
    VDP_setTextPalette(PAL0);
    drawTextSv(cities[currentCity], 15, 11);
}

static void startCity(void)
{
    initBuildings();
    player.x = 16;
    player.y = (FLOOR_Y - 6) * 8;
    player.vy = 0;
    player.dir = 1;
    player.monster = selectedMonster;
    player.punching = FALSE;
    player.punchTimer = 0;
    player.grounded = TRUE;
    clearAll();
    drawSkyline();
    drawBuildings();
    drawHud();
}

static void damageBuildings(void)
{
    const s16 px = player.x / 8;
    const s16 reach = px + (player.dir > 0 ? 5 : -1);

    for (u8 i = 0; i < MAX_BUILDINGS; i++)
    {
        Building *b = &buildings[i];
        if (!b->alive) continue;
        if (reach >= b->x && reach < b->x + b->w && (player.y / 8) + 3 >= b->y)
        {
            if (b->damage < b->h - 2)
            {
                b->damage++;
                score += 10;
                playTone(80 + (b->damage * 4), 9);
            }
            else
            {
                b->alive = FALSE;
                score += 100;
                playTone(32, 14);
            }
            drawBuildings();
            drawHud();
            return;
        }
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

static void updatePlayer(u16 joy, u16 pressed)
{
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
    if ((joy & BUTTON_UP) && player.y > 40)
    {
        player.y -= 1;
    }
    if ((pressed & BUTTON_A) && player.grounded)
    {
        player.vy = -8;
        player.grounded = FALSE;
        playTone(210, 5);
    }
    if ((pressed & (BUTTON_B | BUTTON_C)) != 0)
    {
        player.punching = TRUE;
        player.punchTimer = 10;
        damageBuildings();
    }

    if (!player.grounded)
    {
        player.y += player.vy;
        player.vy++;
        if (player.y >= (FLOOR_Y - 6) * 8)
        {
            player.y = (FLOOR_Y - 6) * 8;
            player.vy = 0;
            player.grounded = TRUE;
        }
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
    drawBuildings();
    drawHud();
    drawMonsterBody(player.x / 8, player.y / 8, player.monster, TRUE, player.punching);
}

static void drawClear(void)
{
    clearAll();
    drawSkyline();
    VDP_fillTileMapRect(BG_A, attr(PAL0, TILE_DARK), 6, 9, 28, 6);
    VDP_setTextPalette(PAL0);
    drawTextSv("STADEN ÄR RIVEN", 12, 10);
    drawTextSv("START FÖR NÄSTA", 12, 12);
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
    PSG_reset();
    SYS_enableInts();

    drawTitle();

    while (TRUE)
    {
        const u16 joy = JOY_readJoypad(JOY_1);
        handleState(joy);

        if ((frame & 7) == 0) stopTone();
        frame++;
        SYS_doVBlankProcess();
    }

    return 0;
}
