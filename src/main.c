#include <genesis.h>
#include <psg.h>
#include "resources.h"
#include "stage_tiles.h"

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
#define PLAYER_SPRITE_FRAMES 16
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
#define POSE_WALK_C 14
#define POSE_WALK_D 15
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
#define TILE_SKY_TOP     (TILE_USER_INDEX + 20)
#define TILE_SKY_MID     (TILE_USER_INDEX + 21)
#define TILE_SKY_LOW     (TILE_USER_INDEX + 22)
#define TILE_CLOUD       (TILE_USER_INDEX + 23)
#define TILE_FAR_TOWER   (TILE_USER_INDEX + 24)
#define TILE_FAR_LIT     (TILE_USER_INDEX + 25)
#define TILE_FACADE_GREY (TILE_USER_INDEX + 26)
#define TILE_FACADE_BRICK (TILE_USER_INDEX + 27)
#define TILE_FACADE_DARK (TILE_USER_INDEX + 28)
#define TILE_ROOF        (TILE_USER_INDEX + 29)
#define TILE_DOOR        (TILE_USER_INDEX + 30)
#define TILE_SIDEWALK    (TILE_USER_INDEX + 31)
#define TILE_CURB        (TILE_USER_INDEX + 32)
#define TILE_ROAD_DARK   (TILE_USER_INDEX + 33)
#define TILE_STREET_LAMP (TILE_USER_INDEX + 34)
#define TILE_FACADE_GLASS (TILE_USER_INDEX + 35)
#define TILE_FACADE_CONCRETE_DARK (TILE_USER_INDEX + 36)
#define TILE_FACADE_BRICK_LIGHT (TILE_USER_INDEX + 37)
#define TILE_SHOP_FRONT  (TILE_USER_INDEX + 38)
#define TILE_ROOF_DARK   (TILE_USER_INDEX + 39)
#define TILE_ROOF_CAP    (TILE_USER_INDEX + 40)
#define TILE_WIN_LIT     (TILE_USER_INDEX + 41)
#define TILE_WIN_BROKEN  (TILE_USER_INDEX + 42)
#define TILE_DAMAGE_SMALL (TILE_USER_INDEX + 43)
#define TILE_DAMAGE_HOLE (TILE_USER_INDEX + 44)
#define TILE_DAMAGE_RUBBLE (TILE_USER_INDEX + 45)
#define TILE_AIRCON      (TILE_USER_INDEX + 46)
#define TILE_SIGN        (TILE_USER_INDEX + 47)
#define TILE_PIPE        (TILE_USER_INDEX + 48)
#define TILE_WINDOW_PERSON_A (TILE_USER_INDEX + 49)
#define TILE_WINDOW_PERSON_B (TILE_USER_INDEX + 50)
#define TILE_WINDOW_PANIC_A  (TILE_USER_INDEX + 51)
#define TILE_WINDOW_PANIC_B  (TILE_USER_INDEX + 52)
#define TILE_TORSO_WHITE     (TILE_USER_INDEX + 53)
#define TILE_TORSO_GLASS     (TILE_USER_INDEX + 54)
#define TILE_TORSO_RIB       (TILE_USER_INDEX + 55)
#define TILE_TORSO_RIB_BACK  (TILE_USER_INDEX + 56)
#define TILE_WATER_BRICK     (TILE_USER_INDEX + 57)
#define TILE_WATER_BAND      (TILE_USER_INDEX + 58)
#define TILE_WATER_ARCH      (TILE_USER_INDEX + 59)
#define TILE_WATER_ROOF_LEFT (TILE_USER_INDEX + 60)
#define TILE_WATER_ROOF_MID  (TILE_USER_INDEX + 61)
#define TILE_WATER_ROOF_RIGHT (TILE_USER_INDEX + 62)
#define TILE_WATER_SPIRE     (TILE_USER_INDEX + 63)
#define TILE_MUSHROOM_CAP_LEFT (TILE_USER_INDEX + 64)
#define TILE_MUSHROOM_CAP_MID (TILE_USER_INDEX + 65)
#define TILE_MUSHROOM_CAP_RIGHT (TILE_USER_INDEX + 66)
#define TILE_MUSHROOM_UNDERSIDE (TILE_USER_INDEX + 67)
#define TILE_MUSHROOM_STEM    (TILE_USER_INDEX + 68)
#define TILE_MUSHROOM_STEM_SHADE (TILE_USER_INDEX + 69)
#define TILE_MUSHROOM_ANTENNA (TILE_USER_INDEX + 70)

typedef enum
{
    STATE_TITLE,
    STATE_LEVEL_SELECT,
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
static u8 selectedCity = 0;
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
    VDP_loadTileData(tileSkyTop, TILE_SKY_TOP, 1, DMA);
    VDP_loadTileData(tileSkyMid, TILE_SKY_MID, 1, DMA);
    VDP_loadTileData(tileSkyLow, TILE_SKY_LOW, 1, DMA);
    VDP_loadTileData(tileFarTower, TILE_FAR_TOWER, 1, DMA);
    VDP_loadTileData(tileFarLit, TILE_FAR_LIT, 1, DMA);
    VDP_loadTileData(tileFacadeGrey, TILE_FACADE_GREY, 1, DMA);
    VDP_loadTileData(tileFacadeBrick, TILE_FACADE_BRICK, 1, DMA);
    VDP_loadTileData(tileFacadeDark, TILE_FACADE_DARK, 1, DMA);
    VDP_loadTileData(tileRoof, TILE_ROOF, 1, DMA);
    VDP_loadTileData(tileDoor, TILE_DOOR, 1, DMA);
    VDP_loadTileData(tileSidewalk, TILE_SIDEWALK, 1, DMA);
    VDP_loadTileData(tileCurb, TILE_CURB, 1, DMA);
    VDP_loadTileData(tileRoadDark, TILE_ROAD_DARK, 1, DMA);
    VDP_loadTileData(tileStreetLamp, TILE_STREET_LAMP, 1, DMA);
    VDP_loadTileData(tileFacadeGlass, TILE_FACADE_GLASS, 1, DMA);
    VDP_loadTileData(tileFacadeConcreteDark, TILE_FACADE_CONCRETE_DARK, 1, DMA);
    VDP_loadTileData(tileFacadeBrickLight, TILE_FACADE_BRICK_LIGHT, 1, DMA);
    VDP_loadTileData(tileShopFront, TILE_SHOP_FRONT, 1, DMA);
    VDP_loadTileData(tileRoofDark, TILE_ROOF_DARK, 1, DMA);
    VDP_loadTileData(tileRoofCap, TILE_ROOF_CAP, 1, DMA);
    VDP_loadTileData(tileWinLit, TILE_WIN_LIT, 1, DMA);
    VDP_loadTileData(tileWinBroken, TILE_WIN_BROKEN, 1, DMA);
    VDP_loadTileData(tileDamageSmall, TILE_DAMAGE_SMALL, 1, DMA);
    VDP_loadTileData(tileDamageHole, TILE_DAMAGE_HOLE, 1, DMA);
    VDP_loadTileData(tileDamageRubble, TILE_DAMAGE_RUBBLE, 1, DMA);
    VDP_loadTileData(tileAircon, TILE_AIRCON, 1, DMA);
    VDP_loadTileData(tileSign, TILE_SIGN, 1, DMA);
    VDP_loadTileData(tilePipe, TILE_PIPE, 1, DMA);
    VDP_loadTileData(tileWindowPersonA, TILE_WINDOW_PERSON_A, 1, DMA);
    VDP_loadTileData(tileWindowPersonB, TILE_WINDOW_PERSON_B, 1, DMA);
    VDP_loadTileData(tileWindowPanicA, TILE_WINDOW_PANIC_A, 1, DMA);
    VDP_loadTileData(tileWindowPanicB, TILE_WINDOW_PANIC_B, 1, DMA);
    VDP_loadTileData(tileTorsoWhite, TILE_TORSO_WHITE, 1, DMA);
    VDP_loadTileData(tileTorsoGlass, TILE_TORSO_GLASS, 1, DMA);
    VDP_loadTileData(tileTorsoRib, TILE_TORSO_RIB, 1, DMA);
    VDP_loadTileData(tileTorsoRibBack, TILE_TORSO_RIB_BACK, 1, DMA);
    VDP_loadTileData(tileWaterBrick, TILE_WATER_BRICK, 1, DMA);
    VDP_loadTileData(tileWaterBand, TILE_WATER_BAND, 1, DMA);
    VDP_loadTileData(tileWaterArch, TILE_WATER_ARCH, 1, DMA);
    VDP_loadTileData(tileWaterRoofLeft, TILE_WATER_ROOF_LEFT, 1, DMA);
    VDP_loadTileData(tileWaterRoofMid, TILE_WATER_ROOF_MID, 1, DMA);
    VDP_loadTileData(tileWaterRoofRight, TILE_WATER_ROOF_RIGHT, 1, DMA);
    VDP_loadTileData(tileWaterSpire, TILE_WATER_SPIRE, 1, DMA);
    VDP_loadTileData(tileMushroomCapLeft, TILE_MUSHROOM_CAP_LEFT, 1, DMA);
    VDP_loadTileData(tileMushroomCapMid, TILE_MUSHROOM_CAP_MID, 1, DMA);
    VDP_loadTileData(tileMushroomCapRight, TILE_MUSHROOM_CAP_RIGHT, 1, DMA);
    VDP_loadTileData(tileMushroomUnderside, TILE_MUSHROOM_UNDERSIDE, 1, DMA);
    VDP_loadTileData(tileMushroomStem, TILE_MUSHROOM_STEM, 1, DMA);
    VDP_loadTileData(tileMushroomStemShade, TILE_MUSHROOM_STEM_SHADE, 1, DMA);
    VDP_loadTileData(tileMushroomAntenna, TILE_MUSHROOM_ANTENNA, 1, DMA);
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

static bool hasActiveHelicopter(void)
{
    for (u8 i = 0; i < MAX_HELIS; i++)
    {
        if (helis[i].active) return TRUE;
    }
    return FALSE;
}

static void playRotorTone(void)
{
    PSG_setTone(1, 82 + ((frame >> 3) & 3));
    PSG_setEnvelope(1, 12);
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
    const u8 farX[10] = {0, 3, 7, 11, 16, 21, 25, 29, 34, 37};
    const u8 farW[10] = {3, 4, 3, 5, 4, 3, 5, 4, 3, 3};
    const u8 farH[10] = {7, 11, 9, 13, 8, 10, 12, 9, 14, 8};

    VDP_fillTileMapRect(BG_B, attr(PAL0, TILE_SKY_TOP), 0, HUD_TILES_H, SCREEN_TILES_W, 5);
    VDP_fillTileMapRect(BG_B, attr(PAL0, TILE_SKY_MID), 0, HUD_TILES_H + 5, SCREEN_TILES_W, 6);
    VDP_fillTileMapRect(BG_B, attr(PAL0, TILE_SKY_LOW), 0, HUD_TILES_H + 11, SCREEN_TILES_W, SCREEN_TILES_H - (HUD_TILES_H + 11));

    for (u8 i = 0; i < 10; i++)
    {
        const u8 x = farX[i];
        const u8 h = farH[i];
        const u8 w = farW[i];
        VDP_fillTileMapRect(BG_B, attr(PAL0, TILE_FAR_TOWER), x, FLOOR_Y - h - 2, w, h + 2);
        for (u8 yy = 1; yy < h; yy += 2)
        {
            for (u8 xx = (i + yy) & 1; xx < w; xx += 2)
            {
                if (((xx + yy + i) & 3) == 0) VDP_setTileMapXY(BG_B, attr(PAL0, TILE_FAR_LIT), x + xx, FLOOR_Y - h - 2 + yy);
            }
        }
        if ((i & 1) == 0)
        {
            VDP_setTileMapXY(BG_B, attr(PAL0, TILE_RED), x + (w / 2), FLOOR_Y - h - 3);
        }
    }

    for (u8 x = 1; x < SCREEN_TILES_W; x += 7)
    {
        VDP_setTileMapXY(BG_B, attr(PAL0, TILE_GREEN), x, FLOOR_Y - 1);
    }
    for (u8 x = 4; x < SCREEN_TILES_W; x += 11)
    {
        VDP_setTileMapXY(BG_B, attr(PAL0, TILE_STREET_LAMP), x, FLOOR_Y - 3);
        VDP_setTileMapXY(BG_B, attr(PAL0, TILE_DARK), x, FLOOR_Y - 2);
    }
}

static void drawRoad(void)
{
    VDP_fillTileMapRect(BG_B, attr(PAL0, TILE_ROAD_DARK), 0, FLOOR_Y, SCREEN_TILES_W, 4);
    for (u8 x = 1; x < SCREEN_TILES_W; x += 6)
    {
        VDP_fillTileMapRect(BG_B, attr(PAL0, TILE_LINE), x, FLOOR_Y + 2, 3, 1);
    }
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
        if (currentCity == 0 && i == 1)
        {
            buildings[i].h = 16;
        }
        if (currentCity == 1 && i == 1)
        {
            buildings[i].h = 17;
        }
        if (currentCity == 2 && i == 1)
        {
            buildings[i].h = 16;
        }
        buildings[i].y = FLOOR_Y - hs[i];
        if ((currentCity == 0 || currentCity == 1 || currentCity == 2) && i == 1)
        {
            buildings[i].y = FLOOR_Y - buildings[i].h;
        }
        buildings[i].damage = 0;
        for (u8 d = 0; d < MAX_DAMAGE_MARKS; d++)
        {
            buildings[i].damageX[d] = 0;
            buildings[i].damageY[d] = 0;
        }
        for (u8 p = 0; p < MAX_WINDOW_PEOPLE; p++)
        {
            const u8 col = 1 + ((p & 1) * 2);
            const u8 row = (currentCity == 1 && i == 1) ? 7 + ((p / 2) * 3) : ((currentCity == 2 && i == 1) ? 2 : 1 + ((p / 2) * 2));
            const u8 personCol = (currentCity == 2 && i == 1) ? 1 + p : col;
            buildings[i].personX[p] = buildings[i].x + (personCol < buildings[i].w ? personCol : 1);
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
    (void)phase;

    if (y < HUD_TILES_H) y = HUD_TILES_H;
    if (y >= FLOOR_Y) y = FLOOR_Y - 1;

    for (u8 xx = 0; xx < b->w; xx++)
    {
        const u8 dustPattern = xx + b->x;
        const u8 dust = TILE_DUST_1 + (dustPattern % 3);
        drawDamageTile(b->x + xx, y, dust);
        if (dustPattern & 1) drawDamageTile(b->x + xx, y + 1, TILE_DUST_1 + ((dustPattern + 1) % 3));
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

        const u8 xA = b->x + 1 + (((rel * 2) + b->x) % (b->w - 1));
        const u8 xB = b->x + ((rel + b->x + 2) % b->w);
        drawDamageTile(xA, yy, (rel & 1) ? TILE_CRACK_SMALL : TILE_CRACK_MID);
        if ((b->crackRows > 5) && ((yy + b->x) & 1)) drawDamageTile(xB, yy, TILE_CRACK_SMALL);
    }

}

static void drawBuildingDamageMark(const Building *b, u8 d, u8 visibleH, u8 drawY)
{
    const u8 cx = b->damageX[d];
    const u8 cy = b->damageY[d];
    if (cy < drawY || cy >= drawY + visibleH) return;

    const u8 severity = d > 7 ? 2 : (d > 3 ? 1 : 0);
    const u8 centerTile = severity == 0 ? TILE_DAMAGE_SMALL : (severity == 1 ? TILE_DAMAGE_HOLE : TILE_DAMAGE_RUBBLE);

    drawDamageTile(cx, cy, centerTile);
    if (cx > b->x && (d & 1)) drawDamageTile(cx - 1, cy, severity == 0 ? TILE_CRACK_SMALL : TILE_WIN_BROKEN);
    if (cx + 1 < b->x + b->w && (d & 2)) drawDamageTile(cx + 1, cy, severity == 2 ? TILE_DAMAGE_HOLE : TILE_DAMAGE_SMALL);
    if (cy + 1 < drawY + visibleH) drawDamageTile(cx, cy + 1, severity == 2 ? TILE_DAMAGE_RUBBLE : TILE_WIN_BROKEN);
    if (cy > drawY && (d & 3) == 0) drawDamageTile(cx, cy - 1, severity == 0 ? TILE_CRACK_SMALL : TILE_DAMAGE_SMALL);
}

static void drawBuildingChunks(const Building *b, u8 visibleH, u8 drawY)
{
    for (u8 d = 0; d < b->damage && d < MAX_DAMAGE_MARKS; d++)
        drawBuildingDamageMark(b, d, visibleH, drawY);
}

static u8 windowPersonTile(const Building *b, u8 p)
{
    const bool wave = (((frame >> 4) + p + b->x) & 1) != 0;
    if (b->personEdible[p]) return wave ? TILE_WINDOW_PERSON_B : TILE_WINDOW_PERSON_A;
    return wave ? TILE_WINDOW_PANIC_B : TILE_WINDOW_PANIC_A;
}

static void clearWindowPersonTile(const Building *b, u8 p)
{
    const u8 x = b->personX[p];
    const u8 y = b->personY[p];
    if (y >= HUD_TILES_H && y < FLOOR_Y && x < SCREEN_TILES_W)
        VDP_setTileMapXY(BG_B, attr(PAL0, TILE_WIN_BROKEN), x, y);
}

static u8 buildingStyle(const Building *b)
{
    return ((b->x / 2) + b->w + b->h) & 3;
}

static u8 buildingBodyTile(const Building *b)
{
    switch (buildingStyle(b))
    {
        case 0: return TILE_FACADE_BRICK_LIGHT;
        case 1: return TILE_FACADE_GREY;
        case 2: return TILE_FACADE_GLASS;
        default: return TILE_YELLOW;
    }
}

static u8 buildingAccentTile(const Building *b)
{
    switch (buildingStyle(b))
    {
        case 0: return TILE_FACADE_BRICK;
        case 1: return TILE_FACADE_CONCRETE_DARK;
        case 2: return TILE_FACADE_DARK;
        default: return TILE_FACADE_BRICK_LIGHT;
    }
}

static u8 buildingWindowTile(const Building *b, u8 xx, u8 yy)
{
    const bool broken = ((xx + yy + b->x) & 5) == 0;
    if (broken) return TILE_WIN_BROKEN;
    if ((((xx * 3) + yy + b->x) & 7) == 0) return TILE_WIN_OFF;
    return buildingStyle(b) == 2 ? TILE_WIN_LIT : TILE_WIN_ON;
}

static bool isTurningTorsoBuilding(const Building *b)
{
    return currentCity == 0 && b == &buildings[1];
}

static bool isOldWaterTowerBuilding(const Building *b)
{
    return currentCity == 1 && b == &buildings[1];
}

static bool isMushroomBuilding(const Building *b)
{
    return currentCity == 2 && b == &buildings[1];
}

static s16 climbTopYForBuilding(const Building *b)
{
    const u8 topRow = isOldWaterTowerBuilding(b) ? b->y + 3 : (isMushroomBuilding(b) ? b->y + 5 : b->y);
    return (topRow * 8) - PLAYER_H;
}

static void drawTurningTorsoBuilding(const Building *b, u8 visibleH, u8 drawY)
{
    const s8 offsets[8] = {1, 1, 0, -1, -1, 0, 1, 1};
    const u8 widths[8] = {4, 4, 5, 5, 4, 5, 4, 3};

    for (u8 yy = 0; yy < visibleH; yy++)
    {
        const u8 worldY = drawY + yy;
        const u8 rel = worldY - b->y;
        const u8 band = (rel / 2) & 7;
        const u8 sectionTop = (rel & 1) == 0;
        const s16 left = b->x + offsets[band];
        const u8 torsoW = widths[band];
        const bool ribLeft = band >= 3 && band <= 5;

        if (sectionTop && !b->collapsing)
        {
            VDP_fillTileMapRect(BG_B, attr(PAL0, TILE_ROOF_CAP), left, worldY, torsoW, 1);
            if (ribLeft) VDP_setTileMapXY(BG_B, attr(PAL0, TILE_TORSO_RIB_BACK), left, worldY);
            else VDP_setTileMapXY(BG_B, attr(PAL0, TILE_TORSO_RIB), left + torsoW - 1, worldY);
            continue;
        }

        for (u8 xx = 0; xx < torsoW; xx++)
        {
            u8 tile = TILE_TORSO_WHITE;
            if (ribLeft && xx == 0) tile = TILE_TORSO_RIB_BACK;
            else if (!ribLeft && xx == torsoW - 1) tile = TILE_TORSO_RIB;
            else if (((xx + rel) & 1) == 0) tile = TILE_TORSO_GLASS;
            VDP_setTileMapXY(BG_B, attr(PAL0, tile), left + xx, worldY);
        }
    }
}

static void drawOldWaterTowerBuilding(const Building *b, u8 visibleH, u8 drawY)
{
    for (u8 yy = 0; yy < visibleH; yy++)
    {
        const u8 worldY = drawY + yy;
        const u8 rel = worldY - b->y;

        if (rel == 0)
        {
            if (!b->collapsing && worldY > HUD_TILES_H)
            {
                VDP_setTileMapXY(BG_B, attr(PAL0, TILE_WATER_SPIRE), b->x + 2, worldY - 1);
            }
            VDP_setTileMapXY(BG_B, attr(PAL0, TILE_WATER_ROOF_MID), b->x + 2, worldY);
            continue;
        }
        if (rel == 1)
        {
            VDP_setTileMapXY(BG_B, attr(PAL0, TILE_WATER_ROOF_LEFT), b->x + 1, worldY);
            VDP_setTileMapXY(BG_B, attr(PAL0, TILE_WATER_ROOF_MID), b->x + 2, worldY);
            VDP_setTileMapXY(BG_B, attr(PAL0, TILE_WATER_ROOF_RIGHT), b->x + 3, worldY);
            continue;
        }
        if (rel == 2)
        {
            VDP_setTileMapXY(BG_B, attr(PAL0, TILE_WATER_ROOF_LEFT), b->x, worldY);
            VDP_setTileMapXY(BG_B, attr(PAL0, TILE_WATER_ROOF_MID), b->x + 1, worldY);
            VDP_setTileMapXY(BG_B, attr(PAL0, TILE_WATER_ROOF_MID), b->x + 2, worldY);
            VDP_setTileMapXY(BG_B, attr(PAL0, TILE_WATER_ROOF_MID), b->x + 3, worldY);
            VDP_setTileMapXY(BG_B, attr(PAL0, TILE_WATER_ROOF_RIGHT), b->x + 4, worldY);
            continue;
        }

        VDP_fillTileMapRect(BG_B, attr(PAL0, TILE_WATER_BRICK), b->x, worldY, b->w, 1);

        if (rel == 3 || rel == 6 || rel == 9 || rel == 15)
        {
            VDP_fillTileMapRect(BG_B, attr(PAL0, TILE_WATER_BAND), b->x, worldY, b->w, 1);
            continue;
        }

        if (rel == 4 || rel == 5)
        {
            VDP_setTileMapXY(BG_B, attr(PAL0, TILE_WATER_ARCH), b->x + 1, worldY);
            VDP_setTileMapXY(BG_B, attr(PAL0, TILE_WATER_ARCH), b->x + 2, worldY);
            VDP_setTileMapXY(BG_B, attr(PAL0, TILE_WATER_ARCH), b->x + 3, worldY);
            continue;
        }

        if (rel == visibleH - 1)
        {
            VDP_setTileMapXY(BG_B, attr(PAL0, TILE_DOOR), b->x + 2, worldY);
        }
        else if (rel > 7 && (rel & 1))
        {
            VDP_setTileMapXY(BG_B, attr(PAL0, TILE_WIN_OFF), b->x + 2, worldY);
        }
    }
}

static void drawMushroomBuilding(const Building *b, u8 visibleH, u8 drawY)
{
    const s16 capX = b->x - 2;
    const u8 stemX = b->x + 1;

    for (u8 yy = 0; yy < visibleH; yy++)
    {
        const u8 worldY = drawY + yy;
        const u8 rel = worldY - b->y;

        if (rel == 0)
        {
            if (!b->collapsing && worldY > HUD_TILES_H)
            {
                VDP_setTileMapXY(BG_B, attr(PAL0, TILE_MUSHROOM_ANTENNA), b->x + 2, worldY - 1);
            }
            VDP_setTileMapXY(BG_B, attr(PAL0, TILE_MUSHROOM_CAP_MID), b->x + 2, worldY);
            continue;
        }
        if (rel == 1)
        {
            VDP_setTileMapXY(BG_B, attr(PAL0, TILE_MUSHROOM_CAP_LEFT), b->x + 1, worldY);
            VDP_setTileMapXY(BG_B, attr(PAL0, TILE_MUSHROOM_CAP_MID), b->x + 2, worldY);
            VDP_setTileMapXY(BG_B, attr(PAL0, TILE_MUSHROOM_CAP_RIGHT), b->x + 3, worldY);
            continue;
        }
        if (rel == 2)
        {
            VDP_fillTileMapRect(BG_B, attr(PAL0, TILE_MUSHROOM_CAP_MID), b->x, worldY, b->w, 1);
            continue;
        }
        if (rel == 3)
        {
            VDP_setTileMapXY(BG_B, attr(PAL0, TILE_MUSHROOM_CAP_LEFT), capX, worldY);
            VDP_fillTileMapRect(BG_B, attr(PAL0, TILE_MUSHROOM_CAP_MID), capX + 1, worldY, 7, 1);
            VDP_setTileMapXY(BG_B, attr(PAL0, TILE_MUSHROOM_CAP_RIGHT), capX + 8, worldY);
            continue;
        }
        if (rel == 4)
        {
            VDP_setTileMapXY(BG_B, attr(PAL0, TILE_MUSHROOM_UNDERSIDE), capX, worldY);
            VDP_setTileMapXY(BG_B, attr(PAL0, TILE_MUSHROOM_UNDERSIDE), capX + 1, worldY);
            VDP_setTileMapXY(BG_B, attr(PAL0, TILE_MUSHROOM_UNDERSIDE), capX + 2, worldY);
            VDP_setTileMapXY(BG_B, attr(PAL0, TILE_MUSHROOM_UNDERSIDE), capX + 3, worldY);
            VDP_setTileMapXY(BG_B, attr(PAL0, TILE_MUSHROOM_UNDERSIDE), capX + 4, worldY);
            VDP_setTileMapXY(BG_B, attr(PAL0, TILE_MUSHROOM_UNDERSIDE), capX + 5, worldY);
            VDP_setTileMapXY(BG_B, attr(PAL0, TILE_MUSHROOM_UNDERSIDE), capX + 6, worldY);
            VDP_setTileMapXY(BG_B, attr(PAL0, TILE_MUSHROOM_UNDERSIDE), capX + 7, worldY);
            VDP_setTileMapXY(BG_B, attr(PAL0, TILE_MUSHROOM_UNDERSIDE), capX + 8, worldY);
            continue;
        }

        const u8 stemTile = ((rel + yy) & 1) ? TILE_MUSHROOM_STEM_SHADE : TILE_MUSHROOM_STEM;
        VDP_fillTileMapRect(BG_B, attr(PAL0, stemTile), stemX, worldY, 3, 1);
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

    const u8 bodyTile = buildingBodyTile(b);
    const u8 accentTile = buildingAccentTile(b);
    const u8 roofTile = buildingStyle(b) == 2 ? TILE_ROOF_DARK : TILE_ROOF;

    if (isTurningTorsoBuilding(b))
    {
        drawTurningTorsoBuilding(b, visibleH, drawY);
        drawBuildingChunks(b, visibleH, drawY);
        if (b->cracking) drawBuildingCracks(b, visibleH);
        if (b->collapsing) drawCollapseDust(b, FLOOR_Y - 1, b->collapseRows);

        for (u8 p = 0; p < MAX_WINDOW_PEOPLE; p++)
        {
            if (!b->personAlive[p]) continue;
            if (b->personY[p] < drawY || b->personY[p] >= drawY + visibleH) continue;
            VDP_setTileMapXY(BG_B, attr(PAL0, windowPersonTile(b, p)), b->personX[p], b->personY[p]);
        }
        return;
    }

    if (isOldWaterTowerBuilding(b))
    {
        drawOldWaterTowerBuilding(b, visibleH, drawY);
        drawBuildingChunks(b, visibleH, drawY);
        if (b->cracking) drawBuildingCracks(b, visibleH);
        if (b->collapsing) drawCollapseDust(b, FLOOR_Y - 1, b->collapseRows);

        for (u8 p = 0; p < MAX_WINDOW_PEOPLE; p++)
        {
            if (!b->personAlive[p]) continue;
            if (b->personY[p] < drawY || b->personY[p] >= drawY + visibleH) continue;
            VDP_setTileMapXY(BG_B, attr(PAL0, windowPersonTile(b, p)), b->personX[p], b->personY[p]);
        }
        return;
    }

    if (isMushroomBuilding(b))
    {
        drawMushroomBuilding(b, visibleH, drawY);
        drawBuildingChunks(b, visibleH, drawY);
        if (b->cracking) drawBuildingCracks(b, visibleH);
        if (b->collapsing) drawCollapseDust(b, FLOOR_Y - 1, b->collapseRows);

        for (u8 p = 0; p < MAX_WINDOW_PEOPLE; p++)
        {
            if (!b->personAlive[p]) continue;
            if (b->personY[p] < drawY || b->personY[p] >= drawY + visibleH) continue;
            VDP_setTileMapXY(BG_B, attr(PAL0, windowPersonTile(b, p)), b->personX[p], b->personY[p]);
        }
        return;
    }

    VDP_fillTileMapRect(BG_B, attr(PAL0, bodyTile), b->x, drawY, b->w, visibleH);
    VDP_fillTileMapRect(BG_B, attr(PAL0, roofTile), b->x, drawY, b->w, 1);
    if (!b->collapsing) VDP_fillTileMapRect(BG_B, attr(PAL0, TILE_ROOF_CAP), b->x, drawY, b->w, 1);
    if (!b->collapsing && visibleH > 2) VDP_fillTileMapRect(BG_B, attr(PAL0, TILE_RED), b->x + 1, drawY + visibleH - 2, b->w - 2, 1);
    for (u8 yy = 3; yy + 1 < visibleH; yy += 3)
    {
        VDP_fillTileMapRect(BG_B, attr(PAL0, accentTile), b->x + 1, drawY + yy, b->w - 1, 1);
    }
    for (u8 yy = 1; yy < visibleH; yy += 2)
    {
        for (u8 xx = 1; xx + 1 < b->w; xx += 2)
        {
            const u8 winTile = buildingWindowTile(b, xx, yy);
            VDP_setTileMapXY(BG_B, attr(PAL0, winTile), b->x + xx, drawY + yy);
            if (xx + 1 < b->w && winTile != TILE_WIN_BROKEN && (((xx * 3) + yy + b->x) & 7) == 0)
            {
                VDP_setTileMapXY(BG_B, attr(PAL0, TILE_AIRCON), b->x + xx + 1, drawY + yy);
            }
        }
    }
    if (!b->collapsing && visibleH > 5 && b->w > 3)
    {
        VDP_setTileMapXY(BG_B, attr(PAL0, buildingStyle(b) == 2 ? TILE_SHOP_FRONT : TILE_DOOR), b->x + 1, drawY + visibleH - 1);
        VDP_setTileMapXY(BG_B, attr(PAL0, buildingStyle(b) == 0 ? TILE_SIGN : TILE_SHOP_FRONT), b->x + 2, drawY + visibleH - 1);
    }
    if (!b->collapsing && visibleH > 7 && b->w > 4)
    {
        VDP_setTileMapXY(BG_B, attr(PAL0, TILE_PIPE), b->x + b->w - 2, drawY + 2);
        VDP_setTileMapXY(BG_B, attr(PAL0, TILE_PIPE), b->x + b->w - 2, drawY + 3);
    }
    drawBuildingChunks(b, visibleH, drawY);
    if (b->cracking) drawBuildingCracks(b, visibleH);
    if (b->collapsing) drawCollapseDust(b, FLOOR_Y - 1, b->collapseRows);

    for (u8 p = 0; p < MAX_WINDOW_PEOPLE; p++)
    {
        if (!b->personAlive[p]) continue;
        if (b->personY[p] < drawY || b->personY[p] >= drawY + visibleH) continue;
        VDP_setTileMapXY(BG_B, attr(PAL0, windowPersonTile(b, p)), b->personX[p], b->personY[p]);
    }
}

static void drawBuildings(void)
{
    drawSkyline();
    for (u8 i = 0; i < MAX_BUILDINGS; i++) drawBuilding(&buildings[i]);
    drawRoad();
}

static void updateWindowPeopleTiles(void)
{
    for (u8 i = 0; i < MAX_BUILDINGS; i++)
    {
        const Building *b = &buildings[i];
        if (!b->alive || b->collapsing) continue;

        for (u8 p = 0; p < MAX_WINDOW_PEOPLE; p++)
        {
            if (!b->personAlive[p]) continue;
            VDP_setTileMapXY(BG_B, attr(PAL0, windowPersonTile(b, p)), b->personX[p], b->personY[p]);
        }
    }
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
    else if (player.landTimer > 0) spriteFrame += (player.landTimer & 4) ? POSE_WALK_A : POSE_WALK_C;
    else if (player.walking) spriteFrame += POSE_WALK_A + ((frame >> 2) & 3);
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
            SPR_setFrame(e->sprite, (frame >> 2) & 3);
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
            SPR_setFrame(t->sprite, (frame >> 3) & 3);
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
            SPR_setFrame(h->sprite, (frame >> 2) & 3);
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
            SPR_setFrame(p->sprite, p->falling ? 0 : ((frame >> 2) & 3));
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

static void drawLevelSelect(void)
{
    char levelNumber[4];

    hidePlayerSprite();
    hideThreatSprites();
    clearAll();
    drawSkyline();
    drawPanel(6, 7, 28, 11);
    VDP_setTextPalette(PAL0);
    drawTextSv("HEMLIG LEVEL SELECT", 10, 9);
    VDP_drawText("<", 9, 12);
    VDP_drawText(">", 30, 12);
    VDP_fillTileMapRect(BG_A, 0, 11, 12, 18, 2);
    VDP_drawText("NIVA", 13, 12);
    intToStr(selectedCity + 1, levelNumber, 2);
    VDP_drawText(levelNumber, 18, 12);
    drawTextSv(cities[selectedCity], 15, 13);
    VDP_drawText("START", 17, 15);
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
        clearWindowPersonTile(b, p);
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
        clearWindowPersonTile(b, p);
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
        drawBuildingDamageMark(b, mark, b->h, b->y);
    }
    else
    {
        b->cracking = TRUE;
        b->crackTimer = BUILDING_CRACK_STEP_FRAMES;
        b->crackRows = 1;
        score += 100;
        playTone(54, 14);
        drawBuildings();
    }
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

static bool damageMushroomHatFromBelow(Building *b, AttackBox attack)
{
    if (!isMushroomBuilding(b)) return FALSE;
    if (attack.yDir >= 0) return FALSE;

    const s16 capX = (b->x - 2) * 8;
    const s16 capY = b->y * 8;
    const s16 capW = 9 * 8;
    const s16 capH = 5 * 8;
    if (!rectsOverlap(attack.x, attack.y, attack.w, attack.h, capX, capY, capW, capH)) return FALSE;

    s16 rawX = (attack.x + (attack.w / 2)) / 8;
    if (rawX < b->x - 2) rawX = b->x - 2;
    if (rawX > b->x + 6) rawX = b->x + 6;
    applyBuildingDamage(b, (u8)rawX, b->y + 3);
    return TRUE;
}

static void damageBuildings(ClimbContact contact, AttackBox attack)
{
    if (!contact.active) return;
    if (contact.building >= MAX_BUILDINGS) return;

    Building *b = &buildings[contact.building];
    if (!b->alive) return;
    if (b->cracking) return;
    if (b->collapsing) return;
    if (damageMushroomHatFromBelow(b, attack)) return;
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
        if (isOldWaterTowerBuilding(b) || isMushroomBuilding(b)) continue;
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
        s16 bLeft = b->x * 8;
        s16 bRight = (b->x + b->w) * 8;
        s16 bTop = b->y * 8;
        const s16 bBottom = FLOOR_Y * 8;
        if (!b->alive) continue;
        if (b->collapsing) continue;

        if (isMushroomBuilding(b))
        {
            bLeft = (b->x + 1) * 8;
            bRight = (b->x + 4) * 8;
            bTop = (b->y + 5) * 8;
        }

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
        const s16 climbTopY = climbTopYForBuilding(b);

        player.y -= 2;
        player.climbPose = POSE_CLIMB_UP;
        if (player.y <= climbTopY)
        {
            const s16 bLeft = b->x * 8;
            const s16 bRight = (b->x + b->w) * 8;
            player.y = climbTopY;
            player.vy = 0;
            climbing = FALSE;
            if (isOldWaterTowerBuilding(b) || isMushroomBuilding(b))
            {
                player.x = climbContact.snapX;
                player.grounded = FALSE;
                onFacade = TRUE;
                onRoof = FALSE;
            }
            else
            {
                player.x = climbContact.attackDir > 0 ? bLeft - 20 : bRight - 28;
                player.grounded = TRUE;
                onFacade = FALSE;
                onRoof = TRUE;
            }
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
            if ((pressed & BUTTON_START) && ((joy & (BUTTON_A | BUTTON_B | BUTTON_C)) == (BUTTON_A | BUTTON_B | BUTTON_C)))
            {
                selectedCity = currentCity;
                state = STATE_LEVEL_SELECT;
                loadGameVideo();
                drawLevelSelect();
                playTone(72, 12);
            }
            else if (pressed & BUTTON_START)
            {
                state = STATE_SELECT;
                drawSelect();
                playTone(120, 8);
            }
            break;

        case STATE_LEVEL_SELECT:
            if (pressed & BUTTON_LEFT)
            {
                selectedCity = (selectedCity + (sizeof(cities) / sizeof(cities[0])) - 1) % (sizeof(cities) / sizeof(cities[0]));
                drawLevelSelect();
                playTone(180, 4);
            }
            if (pressed & BUTTON_RIGHT)
            {
                selectedCity = (selectedCity + 1) % (sizeof(cities) / sizeof(cities[0]));
                drawLevelSelect();
                playTone(160, 4);
            }
            if (pressed & BUTTON_START)
            {
                currentCity = selectedCity;
                state = STATE_INTRO;
                drawIntro();
                playTone(100, 10);
            }
            else if (pressed & (BUTTON_B | BUTTON_C))
            {
                state = STATE_TITLE;
                drawTitle();
                playTone(90, 6);
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
            if (((frame & 7) == 0) && hasActiveHelicopter()) playRotorTone();
            updateBuildingCollapse();
            if ((frame & 15) == 0) updateWindowPeopleTiles();
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
