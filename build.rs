use cmake::Config;

fn main() {
    // build Clipper2
    let mut cmake_builder = Config::new("Clipper2/CPP");
    cmake_builder.define("CLIPPER2_UTILS", "OFF");
    cmake_builder.define("CLIPPER2_EXAMPLES", "OFF");
    cmake_builder.define("CLIPPER2_TESTS", "OFF");
    cmake_builder.define("CLIPPER2_USINGZ", "OFF");
    cmake_builder.define("BUILD_SHARED_LIBS", "OFF");

    #[cfg(target_os = "windows")]
    cmake_builder.cxxflag("/EHsc");

    let dst = cmake_builder.build();

    // build wrapper
    // 需要在系统环境变量中建两个变量：Library和Lib，打开visual studio，新建一个空项目，然后在项目属性>文件目录中复制这两个变量的内容
    let mut cc_builder = cc::Build::new();
    cc_builder.file("src/wrapper.cpp").cpp(true).std("c++17");

    #[cfg(target_os = "windows")]
    cc_builder.flag("-EHsc");

    #[cfg(all(target_os = "windows", debug_assertions))]
    {
        // build.flag("-g");
        // https://learn.microsoft.com/zh-cn/cpp/build/reference/z7-zi-zi-debug-information-format?view=msvc-170
        // Zi 选项生成一个单独的 PDB 文件，其中包含供调试器使用的所有符号化调试信息
        // 使用 /Zi 不会影响优化。 但是，/Zi 的确表示 /debug
        cc_builder.flag("-Zi");
        // https://learn.microsoft.com/zh-cn/cpp/standard-library/iterator-debug-level?view=msvc-170
        // 启用迭代器调试；经过检查的迭代器不相关
        // Clipper2.lib使用了迭代器，所以这里也要开启, 否则link失败
        // 原则上，开启-Zi会默认开启-D_ITERATOR_DEBUG_LEVEL=2，但是这里不知道为什么不行，所以手动开启
        cc_builder.flag("-D_ITERATOR_DEBUG_LEVEL=2");
        // https://learn.microsoft.com/zh-cn/cpp/build/reference/md-mt-ld-use-run-time-library?view=msvc-170
        // 定义 _DEBUG、_MT 和 _DLL，并使此应用程序使用特定于多线程和 DLL 的调试版本的运行库。 它还会让编译器将库名称 MSVCRTD.lib 放入 .obj 文件中。
        // 由于Clipper2.lib是使用MDd编译的，所以这里也要开启, 否则link失败
        // 不太清楚为什么要使用 MDd，而不是 MTd，但是这里也是参考Clipper2
        cc_builder.flag("-MDd");
    }
    cc_builder.include("Clipper2/CPP/Clipper2Lib/include");
    cc_builder.compile("clipper2wrap");

    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-lib=static=Clipper2");
    println!("cargo:rustc-link-lib=static=clipper2wrap");

    #[cfg(target_os = "windows")]
    {
        println!("cargo:rustc-link-lib=dylib=msvcrt");

        // https://learn.microsoft.com/en-us/cpp/porting/upgrade-your-code-to-the-universal-crt?view=msvc-170
        // The Microsoft C Runtime Library (CRT) was refactored in Visual Studio 2015. The Standard C Library, POSIX extensions and Microsoft-specific functions, macros, and global variables were moved into a new library, the Universal C Runtime Library (Universal CRT or UCRT).
        // The UCRT is now a Windows component, and ships as part of Windows 10 and later.
        // 也就是说：msvcrt只包含标准库相关部分, 等同于libstdc++，而ucrtd包含了win10特定api
        //注意win10才有ucrt
        #[cfg(debug_assertions)]
        println!("cargo:rustc-link-lib=dylib=ucrtd");
    }

    #[cfg(target_os = "linux")]
    {
        // linux: stdc++
        // debian: sudo apt-get install lib
        // RedHat: sudo yum install libstdc++-static
        // If can't find lib, add this line: println!("cargo:rustc-link-search=native=/path/to/lib")
        println!("cargo:rustc-link-lib=dylib=stdc++");
    }

    #[cfg(target_os = "macos")]
    {
        // After MacOs Catalina, use libc++ instead
        println!("cargo:rustc-link-lib=dylib=c++"); // need to change this if targeting other platforms
    }

    println!("cargo:rerun-if-changed=build.rs");
}
