#include "astronomy.h"

/* Keep C structs and enums behind a fixed, small ABI with explicit status checks. */
int atlas_sky(int year, int month, int day, int hour, int minute, double second,
    double latitude, double longitude, double elevation, double *values, double *global)
{
    const astro_body_t bodies[] = {BODY_SUN, BODY_MOON, BODY_MERCURY, BODY_VENUS,
        BODY_MARS, BODY_JUPITER, BODY_SATURN, BODY_URANUS, BODY_NEPTUNE};
    astro_time_t time = Astronomy_MakeTime(year, month, day, hour, minute, second);
    astro_observer_t observer = Astronomy_MakeObserver(latitude, longitude, elevation);
    int i;
    for (i=0; i<9; ++i) {
        astro_equatorial_t eq = Astronomy_Equator(bodies[i], &time, observer, EQUATOR_OF_DATE, ABERRATION);
        astro_horizon_t airless;
        astro_horizon_t apparent;
        astro_vector_t geo;
        astro_ecliptic_t ecliptic;
        if (eq.status != ASTRO_SUCCESS) return eq.status;
        airless = Astronomy_Horizon(&time, observer, eq.ra, eq.dec, REFRACTION_NONE);
        apparent = Astronomy_Horizon(&time, observer, eq.ra, eq.dec, REFRACTION_NORMAL);
        geo = Astronomy_GeoVector(bodies[i], time, ABERRATION);
        if (geo.status != ASTRO_SUCCESS) return geo.status;
        ecliptic = Astronomy_Ecliptic(geo);
        if (ecliptic.status != ASTRO_SUCCESS) return ecliptic.status;
        values[i*8+0] = airless.azimuth;
        values[i*8+1] = airless.altitude;
        values[i*8+2] = apparent.altitude;
        values[i*8+3] = eq.ra;
        values[i*8+4] = eq.dec;
        values[i*8+5] = eq.dist;
        values[i*8+6] = ecliptic.elon;
        values[i*8+7] = ecliptic.elat;
    }
    {
        astro_vector_t geo_sun = Astronomy_GeoVector(BODY_SUN, time, ABERRATION);
        astro_vector_t sun_of_date = Astronomy_RotateVector(Astronomy_Rotation_EQJ_EQD(&time), geo_sun);
        astro_equatorial_t sun = Astronomy_EquatorFromVector(sun_of_date);
        astro_angle_result_t phase = Astronomy_MoonPhase(time);
        astro_illum_t illumination = Astronomy_Illumination(BODY_MOON, time);
        if (sun.status != ASTRO_SUCCESS) return sun.status;
        if (phase.status != ASTRO_SUCCESS) return phase.status;
        if (illumination.status != ASTRO_SUCCESS) return illumination.status;
        global[0] = sun.ra;
        global[1] = sun.dec;
        global[2] = Astronomy_SiderealTime(&time);
        global[3] = phase.angle;
        global[4] = illumination.phase_fraction;
    }
    return 0;
}
