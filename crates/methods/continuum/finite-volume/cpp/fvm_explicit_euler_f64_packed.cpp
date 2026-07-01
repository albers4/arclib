#include <cstddef>
#include <cstdint>

extern "C" int fvm_explicit_euler_f64_packed(
    void* const* arguments,
    std::size_t argument_count)
{
    if (argument_count != 5) return 1;
    const auto* state = static_cast<const double*>(arguments[0]);
    const auto* rhs = static_cast<const double*>(arguments[1]);
    const double dt = *static_cast<const double*>(arguments[2]);
    auto* destination = static_cast<double*>(arguments[3]);
    const auto cells = *static_cast<const std::int64_t*>(arguments[4]);
    if (cells <= 0) return 2;
    #pragma omp parallel for schedule(static)
    for (std::int64_t cell = 0; cell < cells; ++cell) {
        destination[cell] = state[cell] + dt * rhs[cell];
    }
    return 0;
}
