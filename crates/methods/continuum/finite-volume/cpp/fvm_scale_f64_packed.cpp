#include <cstddef>
#include <cstdint>

extern "C" int fvm_scale_f64_packed(
    void* const* arguments,
    std::size_t argument_count)
{
    if (argument_count != 4) return 1;
    const auto* field = static_cast<const double*>(arguments[0]);
    const double factor = *static_cast<const double*>(arguments[1]);
    auto* destination = static_cast<double*>(arguments[2]);
    const auto cells = *static_cast<const std::int64_t*>(arguments[3]);
    if (cells <= 0) return 2;
    #pragma omp parallel for schedule(static)
    for (std::int64_t cell = 0; cell < cells; ++cell) {
        destination[cell] = field[cell] * factor;
    }
    return 0;
}