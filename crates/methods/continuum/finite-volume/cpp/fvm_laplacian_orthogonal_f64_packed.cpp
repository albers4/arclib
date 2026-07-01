#include <cstddef>
#include <cstdint>

extern "C" int fvm_laplacian_orthogonal_f64_packed(
    void* const* arguments,
    std::size_t argument_count)
{
    if (argument_count != 8) return 1;
    const auto* field = static_cast<const double*>(arguments[0]);
    const auto* owner = static_cast<const std::int32_t*>(arguments[1]);
    const auto* neighbour = static_cast<const std::int32_t*>(arguments[2]);
    const auto* coefficients = static_cast<const double*>(arguments[3]);
    const auto* volumes = static_cast<const double*>(arguments[4]);
    auto* destination = static_cast<double*>(arguments[5]);
    const auto cells = *static_cast<const std::int64_t*>(arguments[6]);
    const auto faces = *static_cast<const std::int64_t*>(arguments[7]);
    if (cells <= 0 || faces < 0) return 2;

    #pragma omp parallel for schedule(static)
    for (std::int64_t cell = 0; cell < cells; ++cell) {
        destination[cell] = 0.0;
    }

    // The first supported FVM topology is an ordered 1D line. Internal
    // faces can therefore be two-coloured by parity: faces in one colour
    // never touch the same cell, so no atomics are needed.
    for (std::int64_t face = 0; face < faces; ++face) {
        const auto o = static_cast<std::int64_t>(owner[face]);
        const auto n = static_cast<std::int64_t>(neighbour[face]);
        if (o < 0 || n < 0 || o >= cells || n >= cells) return 3;
        if (volumes[o] <= 0.0 || volumes[n] <= 0.0) return 4;
    }

    for (std::int64_t colour = 0; colour < 2; ++colour) {
        #pragma omp parallel for schedule(static)
        for (std::int64_t face = colour; face < faces; face += 2) {
            const auto o = static_cast<std::int64_t>(owner[face]);
            const auto n = static_cast<std::int64_t>(neighbour[face]);
            const double flux = coefficients[face] * (field[n] - field[o]);
            destination[o] += flux / volumes[o];
            destination[n] -= flux / volumes[n];
        }
    }
    return 0;
}