// Host-side smoke test for the cxx bridge: proves the generated header,
// staticlib and error mapping work end-to-end without the SFOS SDK.
//
// Run via `task smoke` (or `task test:all`) from the repo root; it hits
// the live API using WASSERSPIEGEL_API / WASSERSPIEGEL_TOKEN from .envrc.

#include <cstdio>
#include <cstdlib>
#include <stdexcept>
#include <string>

#include "wasserspiegel_core.h"

namespace {

std::string requireEnvOrArg(const char *name, int argc, char **argv, int argIndex)
{
    if (argc > argIndex && argv[argIndex] && argv[argIndex][0] != '\0')
        return std::string(argv[argIndex]);
    const char *env = std::getenv(name);
    if (env && env[0] != '\0')
        return std::string(env);
    std::fprintf(stderr, "missing %s: pass it as argument %d or export %s (see .envrc)\n",
                 name, argIndex, name);
    std::exit(2);
}

} // namespace

int main(int argc, char **argv)
{
    const std::string base = requireEnvOrArg("WASSERSPIEGEL_API", argc, argv, 1);
    const std::string token = requireEnvOrArg("WASSERSPIEGEL_TOKEN", argc, argv, 2);
    try {
        auto client = wasserspiegel::new_client(base, token, "/tmp/ws_smoke_cache");
        auto stations = client->fetch_stations();
        std::printf("stations: %zu\n", (size_t) stations.size());
        if (stations.empty()) {
            std::fprintf(stderr, "no stations returned\n");
            return 1;
        }

        auto matches = wasserspiegel::filter_stations(
            rust::Slice<const wasserspiegel::FfiStationSummary>(stations), "MANNHEIM", 5);
        if (matches.empty()) {
            std::fprintf(stderr, "no MANNHEIM match\n");
            return 1;
        }
        const std::string id(matches[0].id);
        std::printf("first match: %s / %s\n", matches[0].name.c_str(),
                    matches[0].water.c_str());

        auto m = client->fetch_station(id);
        std::printf("%s (%s): %.0f %s, 1d %+.1f, 3d %+.1f, 7d %+.1f, %zu pts\n",
                    m.station_name.c_str(), m.water.c_str(), m.current_level,
                    m.unit.c_str(), m.change_1day, m.change_3day, m.change_7day,
                    (size_t) m.history.size());

        auto sliced = wasserspiegel::slice_series(
            rust::Slice<const wasserspiegel::FfiMeasurementPoint>(m.history), 24, 200);
        auto range = wasserspiegel::series_range(
            rust::Slice<const wasserspiegel::FfiMeasurementPoint>(sliced));
        std::printf("24h slice: %zu pts, min %.1f max %.1f\n",
                    (size_t) sliced.size(), range.min, range.max);

        auto cached = client->load_cached_station(id);
        std::printf("cache reload: from_cache=%d level=%.0f\n", cached.from_cache,
                    cached.current_level);
        return 0;
    } catch (const rust::Error &e) {
        std::fprintf(stderr, "rust error: %s\n", e.what());
        return 1;
    } catch (const std::exception &e) {
        std::fprintf(stderr, "error: %s\n", e.what());
        return 1;
    }
}
