
#ifdef WIN32
    #include <stdint.h>
    #define __int64 int64_t
#endif

#if __has_include(<MvCameraControl.h>)
    //#include <CameraParams.h>
    #include <MvCameraControl.h>
    //#include <MVSErrorDefine.h>
    //#include <MvISPErrorDefine.h>
    //#include <MvObsoleteInterfaces.h>
    //#include <MvSdkExport.h>
    //include <ObsoleteCamParams.h>
    //#include <PixelType.h>
#else
    #define MV_OK 0
#endif