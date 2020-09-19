; ModuleID = 'main'
source_filename = "main"

define void @main() {
entry:
  %a = alloca float
  store float 1.200000e+01, float* %a
  ret void
}

define void @_str_add_char({ i8*, i64, i64 }* %0, i8 %1) {
entry:
  %gepheapptr = getelementptr inbounds { i8*, i64, i64 }, { i8*, i64, i64 }* %0, i32 0, i32 0
  %loadheapptr = load i8*, i8** %gepheapptr
  %geplen = getelementptr inbounds { i8*, i64, i64 }, { i8*, i64, i64 }* %0, i32 0, i32 1
  %loadlen = load i64, i64* %geplen
  %gepmaxlen = getelementptr inbounds { i8*, i64, i64 }, { i8*, i64, i64 }* %0, i32 0, i32 2
  %loadmaxlen = load i64, i64* %gepmaxlen
  %cmplen = icmp eq i64 %loadlen, %loadmaxlen
  br i1 %cmplen, label %malloc, label %end

malloc:                                           ; preds = %entry
  %addgrowth = add i64 %loadmaxlen, 16
  store i64 %addgrowth, i64* %gepmaxlen
  %2 = trunc i64 %addgrowth to i32
  %mallocsize = mul i32 %2, ptrtoint (i8* getelementptr (i8, i8* null, i32 1) to i32)
  %malloclen = tail call i8* @malloc(i32 %mallocsize)
  call void @llvm.memcpy.p0i8.p0i8.i64(i8* align 8 %malloclen, i8* align 8 %loadheapptr, i64 %loadlen, i1 false)
  tail call void @free(i8* %loadheapptr)
  store i8* %malloclen, i8** %gepheapptr
  %gepchar = getelementptr i8, i8* %malloclen, i64 %loadlen
  store i8 %1, i8* %gepchar
  br label %end

end:                                              ; preds = %malloc, %entry
  %addlen = add i64 %loadlen, 1
  store i64 %addlen, i64* %geplen
  ret void
}

declare noalias i8* @malloc(i32)

; Function Attrs: argmemonly nounwind willreturn
declare void @llvm.memcpy.p0i8.p0i8.i64(i8* noalias nocapture writeonly, i8* noalias nocapture readonly, i64, i1 immarg) #0

declare void @free(i8*)

attributes #0 = { argmemonly nounwind willreturn }
