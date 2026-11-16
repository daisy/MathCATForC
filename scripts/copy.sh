outputDir="../Builds"
rulesDir="../Example/Rules"
headerDir="../include"
exampleDir="../Example"

if [ -n "$1" ]; then 
    outputDir="${outputDir}/$1"
fi

mkdir -p ${outputDir}
mkdir -p "${exampleDir}"

cp  -r "${rulesDir}" "${outputDir}/"

# Host build → Example/ so local test.c / test.cpp can link and run with -L.
# Prefer release over debug when both exist.
host_lib=""
for candidate in \
    ../target/release/libmathcat_c.dll \
    ../target/release/libmathcat_c.so \
    ../target/release/libmathcat_c.dylib \
    ../target/debug/libmathcat_c.dll \
    ../target/debug/libmathcat_c.so \
    ../target/debug/libmathcat_c.dylib
do
    if [ -f "$candidate" ]; then
        host_lib="$candidate"
        break
    fi
done
if [ -n "$host_lib" ]; then
    cp -f "$host_lib" "${exampleDir}/"
    echo "Copied $host_lib → Example/"
    # Import library / PDB next to the DLL when present (MSVC link from Example/).
    host_dir=$(dirname "$host_lib")
    for extra in libmathcat_c.dll.lib libmathcat_c.dll.exp libmathcat_c.pdb; do
        if [ -f "${host_dir}/${extra}" ]; then
            cp -f "${host_dir}/${extra}" "${exampleDir}/"
        fi
    done
else
    echo "No host libmathcat_c.{dll,so,dylib} found under target/{release,debug}"
fi

cd "../target"
for dir in *; do
    if [ -d "$dir" ] ; then
        echo $dir
        for file in $(find "$dir" -name "*mathcat_c.a" -o -name "*mathcat_c.so" -o -name "*mathcat_c.dll*" -o -name "*libmathcat_c.pdb");
        do
            mkdir "${outputDir}/${dir}"
            mkdir "${outputDir}/${dir}/include/"
            cp  "${headerDir}/mathcat.h" "${outputDir}/${dir}/include/"
            cp  "${headerDir}/mathcat-c.h" "${outputDir}/${dir}/include/"
            mkdir "${outputDir}/${dir}/lib/"
            cp  "$file" "${outputDir}/${dir}/lib/"
            echo $file
        done
    fi
done
cd -

cd ${outputDir}
for dir in *; do
    if [ -d "$dir" ] ; then
		if [ "${dir}" = "aarch64-linux-android" ] ; then
            rm -rf "mathcat-android-arm64"
			mv "aarch64-linux-android" "mathcat-android-arm64"
		elif [ "$dir" = "armv7-linux-androideabi" ] ; then
            rm -rf "mathcat-android-arm"
			mv "armv7-linux-androideabi" "mathcat-android-arm"
		fi
    fi
done
