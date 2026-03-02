#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BioIntent {
    Pharmacogenomics,
    Ancestry,
    Diversity,
    Nutrition,
    VariantAnnotation,
    Literature,
    SingleCell,
    ProteinStructure,
    Reproducibility,
    SequenceAnalysis,
    Unknown,
}

impl BioIntent {
    pub fn from_query(query: &str) -> Self {
        let q = query.to_lowercase();
        
        if q.contains("drug") 
            || q.contains("pharmacogen")
            || q.contains("cyp2d6")
            || q.contains("cyp2c19")
            || q.contains("cpic")
            || q.contains("metabolizer")
            || q.contains("warfarin")
            || q.contains("medication")
        {
            return BioIntent::Pharmacogenomics;
        }
        
        if q.contains("ancestry")
            || q.contains("pca")
            || q.contains("population")
            || q.contains("admixture")
            || q.contains("sgdp")
            || q.contains("genetic ancestry")
        {
            return BioIntent::Ancestry;
        }
        
        if q.contains("diversity")
            || q.contains("equity")
            || q.contains("heim")
            || q.contains("representation")
            || q.contains("fst")
            || q.contains("heterozygosity")
            || q.contains("underrepresented")
        {
            return BioIntent::Diversity;
        }
        
        if q.contains("nutrition")
            || q.contains("nutrigen")
            || q.contains("diet")
            || q.contains("mthfr")
            || q.contains("folate")
            || q.contains("vitamin")
            || q.contains("omega")
            || q.contains("lactose")
            || q.contains("caffeine")
        {
            return BioIntent::Nutrition;
        }
        
        if q.contains("variant")
            || q.contains("vcf")
            || q.contains("vep")
            || q.contains("clinvar")
            || q.contains("gnomad")
            || q.contains("annotation")
        {
            return BioIntent::VariantAnnotation;
        }
        
        if q.contains("literature")
            || q.contains("pubmed")
            || q.contains("paper")
            || q.contains("citation")
            || q.contains("biorxiv")
        {
            return BioIntent::Literature;
        }
        
        if q.contains("single-cell")
            || q.contains("scrna")
            || q.contains("scanpy")
            || q.contains("clustering")
            || q.contains("marker genes")
        {
            return BioIntent::SingleCell;
        }
        
        if q.contains("protein")
            || q.contains("structure")
            || q.contains("alphafold")
            || q.contains("boltz")
            || q.contains("pdb")
        {
            return BioIntent::ProteinStructure;
        }
        
        if q.contains("repro")
            || q.contains("nextflow")
            || q.contains("singularity")
            || q.contains("conda")
        {
            return BioIntent::Reproducibility;
        }
        
        if q.contains("sequence")
            || q.contains("fastq")
            || q.contains("bam")
            || q.contains("alignment")
            || q.contains("trim")
        {
            return BioIntent::SequenceAnalysis;
        }
        
        BioIntent::Unknown
    }
}
